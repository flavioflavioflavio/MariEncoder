//! Localiza o ffmpeg, monta a linha de comando a partir de um preset e roda o encode
//! lendo o progresso de `-progress pipe:1`.

use crate::config::{AudioCodec, Container, Hw, Preset, RateMode, Speed, VideoCodec};
use std::collections::{HashSet, VecDeque};
use std::ffi::OsString;
use std::io::{BufRead, BufReader};
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

const CREATE_NO_WINDOW: u32 = 0x0800_0000;

#[derive(Clone)]
pub struct Tools {
    pub ffmpeg: PathBuf,
    pub ffprobe: Option<PathBuf>,
    pub encoders: Arc<HashSet<String>>,
}

fn quiet(cmd: &mut Command) -> &mut Command {
    cmd.stdin(Stdio::null()).creation_flags(CREATE_NO_WINDOW)
}

/// Troca `%VAR%` pelo valor da variável de ambiente (o registro guarda o PATH assim).
fn expand_env(s: &str) -> String {
    let mut out = String::new();
    let mut rest = s;
    while let Some(start) = rest.find('%') {
        let Some(len) = rest[start + 1..].find('%') else { break };
        let name = &rest[start + 1..start + 1 + len];
        out.push_str(&rest[..start]);
        match std::env::var(name) {
            Ok(value) if !name.is_empty() => out.push_str(&value),
            _ => out.push_str(&rest[start..start + len + 2]),
        }
        rest = &rest[start + len + 2..];
    }
    out.push_str(rest);
    out
}

/// Pastas do PATH do usuário e do sistema, como estão gravadas no registro.
fn registry_path_dirs() -> Vec<PathBuf> {
    use winreg::RegKey;
    use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ};
    let sources = [
        (HKEY_CURRENT_USER, "Environment"),
        (HKEY_LOCAL_MACHINE, r"SYSTEM\CurrentControlSet\Control\Session Manager\Environment"),
    ];
    let mut dirs = Vec::new();
    for (hive, key) in sources {
        let value: Option<String> =
            RegKey::predef(hive).open_subkey_with_flags(key, KEY_READ).ok().and_then(|k| k.get_value("Path").ok());
        if let Some(value) = value {
            dirs.extend(value.split(';').map(str::trim).filter(|d| !d.is_empty()).map(|d| PathBuf::from(expand_env(d))));
        }
    }
    dirs
}

/// Procura ffmpeg.exe: pasta configurada, ao lado do app, depois no PATH.
pub fn locate(cfg_dir: &str) -> Option<Tools> {
    let mut dirs: Vec<PathBuf> = Vec::new();
    let cfg_dir = cfg_dir.trim().trim_matches('"');
    if !cfg_dir.is_empty() {
        let p = PathBuf::from(cfg_dir);
        if p.is_file() {
            if let Some(parent) = p.parent() {
                dirs.push(parent.to_path_buf());
            }
        } else {
            dirs.push(p.join("bin"));
            dirs.push(p);
        }
    }
    if let Some(exe_dir) = std::env::current_exe().ok().and_then(|e| e.parent().map(Path::to_path_buf)) {
        dirs.push(exe_dir.clone());
        dirs.push(exe_dir.join("ffmpeg"));
        dirs.push(exe_dir.join("ffmpeg").join("bin"));
    }
    if let Some(path) = std::env::var_os("PATH") {
        dirs.extend(std::env::split_paths(&path));
    }
    // Aberto pelo menu do Windows 11, o app herda o PATH de um processo do sistema,
    // que pode não ter as entradas do usuário: lê também o PATH gravado no registro.
    dirs.extend(registry_path_dirs());

    let ffmpeg = dirs.iter().map(|d| d.join("ffmpeg.exe")).find(|p| p.is_file())?;
    let ffprobe = ffmpeg
        .parent()
        .map(|d| d.join("ffprobe.exe"))
        .filter(|p| p.is_file())
        .or_else(|| dirs.iter().map(|d| d.join("ffprobe.exe")).find(|p| p.is_file()));

    let mut encoders = HashSet::new();
    if let Ok(out) = quiet(Command::new(&ffmpeg).args(["-hide_banner", "-encoders"])).output() {
        for line in String::from_utf8_lossy(&out.stdout).lines() {
            let mut it = line.split_whitespace();
            if let (Some(flags), Some(name)) = (it.next(), it.next()) {
                if flags.len() == 6 && (flags.starts_with('V') || flags.starts_with('A')) {
                    encoders.insert(name.to_string());
                }
            }
        }
    }
    Some(Tools { ffmpeg, ffprobe, encoders: Arc::new(encoders) })
}

/// Duração do arquivo em segundos (ffprobe; se não houver, lê o cabeçalho pelo ffmpeg).
pub fn probe_duration(tools: &Tools, input: &Path) -> Option<f64> {
    if let Some(probe) = &tools.ffprobe {
        let out = quiet(Command::new(probe).args([
            "-v",
            "error",
            "-show_entries",
            "format=duration",
            "-of",
            "default=noprint_wrappers=1:nokey=1",
        ]))
        .arg(input)
        .output()
        .ok()?;
        if let Ok(d) = String::from_utf8_lossy(&out.stdout).trim().parse::<f64>() {
            if d > 0.0 {
                return Some(d);
            }
        }
    }
    let out = quiet(Command::new(&tools.ffmpeg).arg("-hide_banner").arg("-i").arg(input)).output().ok()?;
    let text = String::from_utf8_lossy(&out.stderr);
    let rest = text.split("Duration: ").nth(1)?;
    let stamp = rest.split(',').next()?.trim();
    let mut secs = 0.0;
    for part in stamp.split(':') {
        secs = secs * 60.0 + part.parse::<f64>().ok()?;
    }
    (secs > 0.0).then_some(secs)
}

fn scale_filter(short_side: u32) -> String {
    let n = (short_side / 2 * 2).max(2);
    // Limita o lado menor a `n` sem ampliar e mantendo dimensões pares.
    format!(
        "scale='if(gt(iw,ih),-2,trunc(min({n},iw)/2)*2)':'if(gt(iw,ih),trunc(min({n},ih)/2)*2,-2)'"
    )
}

fn video_encoder(p: &Preset, hw: Hw, encoders: &HashSet<String>) -> &'static str {
    match (p.video, hw) {
        (VideoCodec::H264, Hw::Cpu) => "libx264",
        (VideoCodec::H264, Hw::Nvidia) => "h264_nvenc",
        (VideoCodec::H264, Hw::Intel) => "h264_qsv",
        (VideoCodec::H264, Hw::Amd) => "h264_amf",
        (VideoCodec::H265, Hw::Cpu) => "libx265",
        (VideoCodec::H265, Hw::Nvidia) => "hevc_nvenc",
        (VideoCodec::H265, Hw::Intel) => "hevc_qsv",
        (VideoCodec::H265, Hw::Amd) => "hevc_amf",
        (VideoCodec::Av1, Hw::Cpu) => {
            if encoders.contains("libsvtav1") || !encoders.contains("libaom-av1") {
                "libsvtav1"
            } else {
                "libaom-av1"
            }
        }
        (VideoCodec::Av1, Hw::Nvidia) => "av1_nvenc",
        (VideoCodec::Av1, Hw::Intel) => "av1_qsv",
        (VideoCodec::Av1, Hw::Amd) => "av1_amf",
        (VideoCodec::Vp9, _) => "libvpx-vp9",
        (VideoCodec::Prores, _) => "prores_ks",
        (VideoCodec::Copy, _) => "copy",
    }
}

/// Bitrate de vídeo (kbps) quando o preset usa bitrate fixo ou tamanho-alvo.
fn target_kbps(p: &Preset, duration: Option<f64>) -> Option<u32> {
    match p.rate_mode {
        RateMode::Quality => None,
        RateMode::Bitrate => Some(p.video_kbps.max(50)),
        RateMode::TargetSize => {
            let d = duration.filter(|d| *d > 0.5)?;
            let audio = match p.audio {
                AudioCodec::None => 0.0,
                AudioCodec::Pcm => 1536.0,
                _ => p.audio_kbps as f64,
            };
            // 3% de folga para o overhead do container.
            let total = p.target_mb as f64 * 8.0 * 1024.0 * 0.97 / d;
            Some((total - audio).max(80.0) as u32)
        }
    }
}

fn push_video(p: &Preset, duration: Option<f64>, force_cpu: bool, encoders: &HashSet<String>, a: &mut Vec<String>) {
    if p.video == VideoCodec::Copy {
        a.extend(["-c:v".into(), "copy".into()]);
        return;
    }
    let mut vf: Vec<String> = Vec::new();
    if p.max_short_side > 0 {
        vf.push(scale_filter(p.max_short_side));
    }
    if p.max_fps > 0 {
        vf.push(format!("fps='min(source_fps,{})'", p.max_fps));
    }
    if !vf.is_empty() {
        a.extend(["-vf".into(), vf.join(",")]);
    }

    let hw = if force_cpu { Hw::Cpu } else { p.hw };
    let enc = video_encoder(p, hw, encoders);
    a.extend(["-c:v".into(), enc.into()]);

    let kbps = target_kbps(p, duration);
    let strict = p.rate_mode == RateMode::TargetSize;
    let crf = p.crf.to_string();
    fn ext(a: &mut Vec<String>, items: &[&str]) {
        a.extend(items.iter().map(|x| x.to_string()));
    }
    let bitrate = |k: u32| -> [String; 6] {
        let max = if strict { k } else { k * 3 / 2 };
        ["-b:v".into(), format!("{k}k"), "-maxrate".into(), format!("{max}k"), "-bufsize".into(), format!("{}k", k * 2)]
    };

    match enc {
        "libx264" | "libx265" => {
            ext(a, &["-preset", match p.speed { Speed::Fast => "fast", Speed::Medium => "medium", Speed::Slow => "slow" }]);
            match kbps {
                Some(k) => a.extend(bitrate(k)),
                None => ext(a, &["-crf", &crf]),
            }
        }
        "libsvtav1" => {
            ext(a, &["-preset", match p.speed { Speed::Fast => "10", Speed::Medium => "8", Speed::Slow => "5" }]);
            match kbps {
                Some(k) => a.extend(["-b:v".into(), format!("{k}k")]),
                None => ext(a, &["-crf", &crf]),
            }
        }
        "libaom-av1" | "libvpx-vp9" => {
            ext(a, &["-row-mt", "1", "-cpu-used", match p.speed { Speed::Fast => "6", Speed::Medium => "4", Speed::Slow => "2" }]);
            match kbps {
                Some(k) => a.extend(["-b:v".into(), format!("{k}k")]),
                None => ext(a, &["-crf", &crf, "-b:v", "0"]),
            }
        }
        "h264_nvenc" | "hevc_nvenc" | "av1_nvenc" => {
            ext(a, &["-preset", match p.speed { Speed::Fast => "p3", Speed::Medium => "p5", Speed::Slow => "p7" }, "-rc", "vbr"]);
            match kbps {
                Some(k) => a.extend(bitrate(k)),
                None => ext(a, &["-cq", &crf, "-b:v", "0"]),
            }
        }
        "h264_qsv" | "hevc_qsv" | "av1_qsv" => {
            ext(a, &["-preset", match p.speed { Speed::Fast => "faster", Speed::Medium => "medium", Speed::Slow => "slower" }]);
            match kbps {
                Some(k) => a.extend(bitrate(k)),
                None => ext(a, &["-global_quality", &crf]),
            }
        }
        "h264_amf" | "hevc_amf" | "av1_amf" => {
            ext(a, &["-quality", match p.speed { Speed::Fast => "speed", Speed::Medium => "balanced", Speed::Slow => "quality" }]);
            match kbps {
                Some(k) => {
                    ext(a, &["-rc", "vbr_peak"]);
                    a.extend(bitrate(k));
                }
                None => ext(a, &["-rc", "cqp", "-qp_i", &crf, "-qp_p", &crf, "-qp_b", &crf]),
            }
        }
        "prores_ks" => ext(a, &["-profile:v", "3", "-vendor", "apl0", "-pix_fmt", "yuv422p10le"]),
        _ => {}
    }

    if enc != "prores_ks" {
        a.extend(["-pix_fmt".into(), "yuv420p".into()]);
    }
    if p.video == VideoCodec::H265 && matches!(p.container, Container::Mp4 | Container::Mov) {
        a.extend(["-tag:v".into(), "hvc1".into()]);
    }
}

fn push_audio(p: &Preset, a: &mut Vec<String>) {
    let kbps = format!("{}k", p.audio_kbps.clamp(16, 512));
    let items: Vec<&str> = match p.audio {
        AudioCodec::Aac => vec!["-c:a", "aac", "-b:a", &kbps],
        AudioCodec::Opus => vec!["-c:a", "libopus", "-b:a", &kbps],
        AudioCodec::Mp3 => vec!["-c:a", "libmp3lame", "-b:a", &kbps],
        AudioCodec::Pcm => vec!["-c:a", "pcm_s16le"],
        AudioCodec::Copy => vec!["-c:a", "copy"],
        AudioCodec::None => vec!["-an"],
    };
    a.extend(items.into_iter().map(String::from));
}

/// Divide argumentos extras respeitando aspas duplas.
fn split_args(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut quoted = false;
    let mut has = false;
    for c in s.chars() {
        match c {
            '"' => {
                quoted = !quoted;
                has = true;
            }
            c if c.is_whitespace() && !quoted => {
                if has {
                    out.push(std::mem::take(&mut cur));
                    has = false;
                }
            }
            c => {
                cur.push(c);
                has = true;
            }
        }
    }
    if has {
        out.push(cur);
    }
    out
}

/// Argumentos entre a entrada e a saída (codec, filtros, taxa...).
pub fn encode_args(p: &Preset, duration: Option<f64>, force_cpu: bool, encoders: &HashSet<String>) -> Vec<String> {
    let mut a: Vec<String> = Vec::new();
    match p.container {
        Container::Gif => {
            let side = if p.max_short_side > 0 { p.max_short_side } else { 480 };
            let fps = if p.max_fps > 0 { p.max_fps } else { 12 };
            a.extend([
                "-vf".into(),
                format!(
                    "fps='min(source_fps,{fps})',{}:flags=lanczos,split[a][b];[a]palettegen=stats_mode=diff[p];[b][p]paletteuse=dither=bayer:bayer_scale=5:diff_mode=rectangle",
                    scale_filter(side)
                ),
                "-loop".into(),
                "0".into(),
                "-an".into(),
            ]);
        }
        c if c.audio_only() => {
            a.push("-vn".into());
            push_audio(p, &mut a);
        }
        _ => {
            push_video(p, duration, force_cpu, encoders, &mut a);
            push_audio(p, &mut a);
            a.extend(["-map_metadata".into(), "0".into()]);
        }
    }
    if matches!(p.container, Container::Mp4 | Container::Mov | Container::M4a) {
        a.extend(["-movflags".into(), "+faststart".into()]);
    }
    a.extend(split_args(&p.extra_args));
    a
}

/// Linha de comando completa para o ffmpeg.
pub fn build_args(
    p: &Preset,
    input: &Path,
    output: &Path,
    duration: Option<f64>,
    force_cpu: bool,
    encoders: &HashSet<String>,
) -> Vec<OsString> {
    let mut a: Vec<OsString> = ["-hide_banner", "-y", "-nostdin", "-i"].iter().map(OsString::from).collect();
    a.push(input.into());
    a.extend(encode_args(p, duration, force_cpu, encoders).into_iter().map(OsString::from));
    a.extend(["-progress", "pipe:1", "-nostats"].iter().map(OsString::from));
    a.push(output.into());
    a
}

/// Comando legível para mostrar no editor de presets.
pub fn preview(p: &Preset, encoders: &HashSet<String>) -> String {
    let args = encode_args(p, None, false, encoders);
    let mid: Vec<String> = args
        .iter()
        .map(|x| if x.contains([' ', '\'', ';', '[']) { format!("\"{x}\"") } else { x.clone() })
        .collect();
    format!("ffmpeg -i entrada {} saida.{}", mid.join(" "), p.container.ext())
}

#[derive(Clone, Copy, Default)]
pub struct Progress {
    pub out_secs: f64,
    pub speed: f32,
    pub out_bytes: u64,
}

pub enum Outcome {
    Done,
    Cancelled,
    Failed(String),
}

/// Mata o processo em andamento (se houver) guardado no slot.
pub fn kill(child_slot: &Mutex<Option<Child>>) {
    if let Some(child) = child_slot.lock().unwrap().as_mut() {
        let _ = child.kill();
    }
}

/// Roda o ffmpeg até o fim. `child_slot` expõe o processo para que outra thread possa cancelar.
pub fn run(
    ffmpeg: &Path,
    args: &[OsString],
    cancel: &AtomicBool,
    child_slot: &Mutex<Option<Child>>,
    mut on_progress: impl FnMut(Progress),
) -> Outcome {
    let mut cmd = Command::new(ffmpeg);
    cmd.args(args).stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped()).creation_flags(CREATE_NO_WINDOW);
    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => return Outcome::Failed(format!("Não foi possível iniciar o ffmpeg: {e}")),
    };
    let stdout = child.stdout.take().expect("stdout piped");
    let stderr = child.stderr.take().expect("stderr piped");
    *child_slot.lock().unwrap() = Some(child);
    if cancel.load(Ordering::SeqCst) {
        kill(child_slot);
    }

    let tail: Arc<Mutex<VecDeque<String>>> = Arc::default();
    let tail_writer = tail.clone();
    let stderr_thread = std::thread::spawn(move || {
        for chunk in BufReader::new(stderr).split(b'\n').map_while(Result::ok) {
            let text = String::from_utf8_lossy(&chunk);
            // Com `\r` o ffmpeg reescreve a mesma linha; fica só o último trecho.
            let line = text.rsplit('\r').find(|s| !s.trim().is_empty()).unwrap_or("").trim_end();
            if line.is_empty() {
                continue;
            }
            let mut t = tail_writer.lock().unwrap();
            if t.len() >= 30 {
                t.pop_front();
            }
            t.push_back(line.to_string());
        }
    });

    let mut p = Progress::default();
    for line in BufReader::new(stdout).lines().map_while(Result::ok) {
        let Some((key, value)) = line.split_once('=') else { continue };
        let value = value.trim();
        match key {
            "out_time_us" => {
                if let Ok(us) = value.parse::<f64>() {
                    p.out_secs = (us / 1e6).max(0.0);
                }
            }
            "speed" => {
                if let Ok(x) = value.trim_end_matches('x').trim().parse::<f32>() {
                    p.speed = x;
                }
            }
            "total_size" => {
                if let Ok(b) = value.parse::<u64>() {
                    p.out_bytes = b;
                }
            }
            "progress" => on_progress(p),
            _ => {}
        }
    }

    let _ = stderr_thread.join();
    let status = child_slot.lock().unwrap().take().map(|mut c| c.wait());
    if cancel.load(Ordering::SeqCst) {
        return Outcome::Cancelled;
    }
    match status {
        Some(Ok(s)) if s.success() => Outcome::Done,
        _ => {
            let log: Vec<String> = tail.lock().unwrap().iter().cloned().collect();
            if log.is_empty() {
                Outcome::Failed("O ffmpeg terminou com erro.".into())
            } else {
                Outcome::Failed(log.join("\n"))
            }
        }
    }
}
