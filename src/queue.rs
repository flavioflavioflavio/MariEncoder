//! Fila de encodes: uma thread de trabalho roda um job por vez e publica o progresso.

use crate::config::{Hw, Preset, VideoCodec};
use crate::ffmpeg::{self, Outcome, Tools};
use eframe::egui;
use std::path::{Path, PathBuf};
use std::process::Child;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::time::Instant;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum JobState {
    /// Na lista, esperando o usuário mandar encodar.
    Idle,
    Queued,
    Running,
    Done,
    Failed,
    Cancelled,
}

#[derive(Clone)]
pub struct Job {
    pub id: u64,
    pub input: PathBuf,
    pub output: Option<PathBuf>,
    pub preset: Preset,
    pub state: JobState,
    /// 0..1; `None` enquanto a duração do arquivo é desconhecida.
    pub fraction: Option<f32>,
    pub speed: f32,
    pub eta_secs: Option<f32>,
    pub in_bytes: u64,
    pub out_bytes: u64,
    pub error: Option<String>,
    pub note: Option<String>,
    cancel: Arc<AtomicBool>,
    child: Arc<Mutex<Option<Child>>>,
}

struct Inner {
    jobs: Vec<Job>,
    next_id: u64,
    tools: Option<Tools>,
    repaint: Option<egui::Context>,
}

#[derive(Clone)]
pub struct Queue {
    shared: Arc<(Mutex<Inner>, Condvar)>,
}

impl Queue {
    pub fn new(tools: Option<Tools>) -> Self {
        let shared = Arc::new((Mutex::new(Inner { jobs: Vec::new(), next_id: 1, tools, repaint: None }), Condvar::new()));
        let queue = Queue { shared };
        let worker = queue.clone();
        std::thread::Builder::new()
            .name("encode-worker".into())
            .spawn(move || worker.work())
            .expect("spawn worker");
        queue
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Inner> {
        self.shared.0.lock().unwrap()
    }

    pub fn set_repaint(&self, ctx: egui::Context) {
        self.lock().repaint = Some(ctx);
    }

    pub fn set_tools(&self, tools: Option<Tools>) {
        self.lock().tools = tools;
    }

    pub fn snapshot(&self) -> Vec<Job> {
        self.lock().jobs.clone()
    }

    pub fn add(&self, files: impl IntoIterator<Item = PathBuf>, preset: &Preset, start: bool) {
        let mut inner = self.lock();
        for input in files {
            let id = inner.next_id;
            inner.next_id += 1;
            let in_bytes = std::fs::metadata(&input).map(|m| m.len()).unwrap_or(0);
            inner.jobs.push(Job {
                id,
                input,
                output: None,
                preset: preset.clone(),
                state: if start { JobState::Queued } else { JobState::Idle },
                fraction: None,
                speed: 0.0,
                eta_secs: None,
                in_bytes,
                out_bytes: 0,
                error: None,
                note: None,
                cancel: Arc::default(),
                child: Arc::default(),
            });
        }
        drop(inner);
        self.shared.1.notify_all();
    }

    fn with_job(&self, id: u64, f: impl FnOnce(&mut Job)) {
        if let Some(job) = self.lock().jobs.iter_mut().find(|j| j.id == id) {
            f(job);
        }
    }

    pub fn set_preset(&self, id: u64, preset: &Preset) {
        self.with_job(id, |j| {
            if j.state == JobState::Idle {
                j.preset = preset.clone();
            }
        });
    }

    /// Aplica o preset a todos os jobs que ainda não começaram.
    pub fn set_preset_all_idle(&self, preset: &Preset) {
        for j in self.lock().jobs.iter_mut().filter(|j| j.state == JobState::Idle) {
            j.preset = preset.clone();
        }
    }

    /// Coloca na fila um job parado, com erro ou cancelado.
    pub fn start(&self, id: u64) {
        self.with_job(id, reset_to_queued);
        self.shared.1.notify_all();
    }

    pub fn start_all_idle(&self) {
        for j in self.lock().jobs.iter_mut().filter(|j| j.state == JobState::Idle) {
            reset_to_queued(j);
        }
        self.shared.1.notify_all();
    }

    pub fn cancel(&self, id: u64) {
        self.with_job(id, cancel_job);
    }

    pub fn cancel_all(&self) {
        for j in self.lock().jobs.iter_mut() {
            cancel_job(j);
        }
    }

    /// Remove da lista (cancelando antes, se estiver rodando).
    pub fn remove(&self, id: u64) {
        let mut inner = self.lock();
        if let Some(pos) = inner.jobs.iter().position(|j| j.id == id) {
            cancel_job(&mut inner.jobs[pos]);
            inner.jobs.remove(pos);
        }
    }

    pub fn clear_finished(&self) {
        self.lock().jobs.retain(|j| !matches!(j.state, JobState::Done | JobState::Cancelled | JobState::Failed));
    }

    fn work(&self) {
        loop {
            // Espera o próximo job na fila.
            let (job, tools, repaint) = {
                let mut inner = self.lock();
                loop {
                    if let Some(j) = inner.jobs.iter_mut().find(|j| j.state == JobState::Queued) {
                        j.state = JobState::Running;
                        j.fraction = None;
                        let job = j.clone();
                        break (job, inner.tools.clone(), inner.repaint.clone());
                    }
                    inner = self.shared.1.wait(inner).unwrap();
                }
            };
            let repaint = move || {
                if let Some(ctx) = &repaint {
                    ctx.request_repaint();
                }
            };
            repaint();

            let (state, error) = match &tools {
                Some(tools) => self.encode(&job, tools, &repaint),
                None => (JobState::Failed, Some("ffmpeg não encontrado. Indique a pasta em Ajustes.".to_string())),
            };
            self.with_job(job.id, |j| {
                j.state = state;
                j.error = error;
                j.eta_secs = None;
                if state == JobState::Done {
                    j.fraction = Some(1.0);
                    if let Some(out) = &j.output {
                        j.out_bytes = std::fs::metadata(out).map(|m| m.len()).unwrap_or(j.out_bytes);
                    }
                }
            });
            repaint();
        }
    }

    fn encode(&self, job: &Job, tools: &Tools, repaint: &dyn Fn()) -> (JobState, Option<String>) {
        if !job.input.is_file() {
            return (JobState::Failed, Some("Arquivo de entrada não encontrado.".into()));
        }
        let output = match output_path(&job.input, &job.preset) {
            Ok(p) => p,
            Err(e) => return (JobState::Failed, Some(format!("Não foi possível criar a pasta de saída: {e}"))),
        };
        self.with_job(job.id, |j| j.output = Some(output.clone()));
        let duration = ffmpeg::probe_duration(tools, &job.input);

        let uses_gpu = job.preset.hw != Hw::Cpu
            && job.preset.has_video_settings()
            && !matches!(job.preset.video, VideoCodec::Copy | VideoCodec::Vp9 | VideoCodec::Prores);
        let mut force_cpu = false;
        loop {
            let args = ffmpeg::build_args(&job.preset, &job.input, &output, duration, force_cpu, &tools.encoders);
            let started = Instant::now();
            let outcome = ffmpeg::run(&tools.ffmpeg, &args, &job.cancel, &job.child, |p| {
                let fraction = duration.map(|d| (p.out_secs / d).clamp(0.0, 1.0) as f32);
                let elapsed = started.elapsed().as_secs_f32();
                self.with_job(job.id, |j| {
                    j.fraction = fraction;
                    j.speed = p.speed;
                    j.out_bytes = p.out_bytes;
                    j.eta_secs = fraction.filter(|f| *f > 0.01 && elapsed > 1.0).map(|f| elapsed * (1.0 - f) / f);
                });
                repaint();
            });
            match outcome {
                Outcome::Done => return (JobState::Done, None),
                Outcome::Cancelled => {
                    let _ = std::fs::remove_file(&output);
                    return (JobState::Cancelled, None);
                }
                Outcome::Failed(_) if uses_gpu && !force_cpu => {
                    // Encoder de GPU indisponível nesta máquina: tenta de novo por software.
                    force_cpu = true;
                    self.with_job(job.id, |j| {
                        j.note = Some("GPU indisponível — encodando pela CPU".into());
                        j.fraction = None;
                    });
                    repaint();
                }
                Outcome::Failed(log) => {
                    let _ = std::fs::remove_file(&output);
                    return (JobState::Failed, Some(log));
                }
            }
        }
    }
}

fn reset_to_queued(j: &mut Job) {
    if matches!(j.state, JobState::Idle | JobState::Failed | JobState::Cancelled) {
        j.state = JobState::Queued;
        j.cancel = Arc::default();
        j.error = None;
        j.note = None;
        j.fraction = None;
        j.out_bytes = 0;
        j.output = None;
    }
}

fn cancel_job(j: &mut Job) {
    match j.state {
        JobState::Running => {
            j.cancel.store(true, Ordering::SeqCst);
            ffmpeg::kill(&j.child);
        }
        JobState::Queued => j.state = JobState::Cancelled,
        _ => {}
    }
}

/// Caminho de saída que nunca sobrescreve um arquivo existente.
pub fn output_path(input: &Path, preset: &Preset) -> std::io::Result<PathBuf> {
    let parent = input.parent().unwrap_or(Path::new(".")).to_path_buf();
    let dir = match preset.output_dir.trim() {
        "" => parent,
        d if Path::new(d).is_absolute() => PathBuf::from(d),
        d => parent.join(d),
    };
    std::fs::create_dir_all(&dir)?;
    let stem = input.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_else(|| "video".into());
    let base = format!("{stem}{}", preset.suffix.trim());
    let ext = preset.container.ext();
    let mut candidate = dir.join(format!("{base}.{ext}"));
    let mut n = 2;
    while candidate.exists() {
        candidate = dir.join(format!("{base} ({n}).{ext}"));
        n += 1;
    }
    Ok(candidate)
}
