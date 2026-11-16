//! Interface: fila de batch, editor de presets e ajustes. No modo compacto (menu
//! contextual) mostra só o progresso e fecha sozinha ao terminar.

use crate::config::{AudioCodec, Config, Container, Hw, Preset, RateMode, Speed, VideoCodec};
use crate::i18n::Language;
use crate::ffmpeg::{self, Tools};
use crate::ipc::{self, Msg};
use crate::queue::{Job, JobState, Queue};
use crate::{shell, theme};
use eframe::egui::{self, Align, Color32, Layout, RichText};
use std::collections::HashSet;
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver};
use std::time::{Duration, Instant};

const FULL_SIZE: [f32; 2] = [960.0, 660.0];
const COMPACT_SIZE: [f32; 2] = [540.0, 300.0];

pub fn run(initial: Msg) {
    let compact = initial.start && !initial.files.is_empty();
    let mut viewport = egui::ViewportBuilder::default()
        .with_title(crate::brand::APP_NAME)
        .with_inner_size(if compact { COMPACT_SIZE } else { FULL_SIZE })
        .with_min_inner_size([440.0, 220.0])
        .with_drag_and_drop(true);
    if let Ok(icon) = eframe::icon_data::from_png_bytes(include_bytes!("../assets/icon.png")) {
        viewport = viewport.with_icon(icon);
    }
    let options = eframe::NativeOptions { viewport, centered: true, ..Default::default() };
    let _ = eframe::run_native(
        crate::brand::APP_NAME,
        options,
        Box::new(move |cc| Ok(Box::new(App::new(cc, initial, compact)))),
    );
}

#[derive(PartialEq, Clone, Copy)]
enum Tab {
    Queue,
    Presets,
    Settings,
    About,
}

struct App {
    cfg: Config,
    /// Estado no frame anterior, para detectar edições e salvar sozinho.
    last_cfg: Config,
    dirty_at: Option<Instant>,
    queue: Queue,
    tools: Option<Tools>,
    rx: Receiver<Msg>,
    tab: Tab,
    compact: bool,
    /// Preset aplicado aos arquivos adicionados pela janela.
    current_preset: String,
    edit_sel: usize,
    about: crate::about::About,
    /// Rodando dentro do pacote da Microsoft Store (o menu vem do manifesto).
    packaged: bool,
    menu_installed: bool,
    /// Pacote do menu principal do Windows 11 registrado.
    modern_installed: bool,
    ext_text: String,
    toast: Option<(String, bool, Instant)>,
    done_at: Option<Instant>,
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>, initial: Msg, compact: bool) -> Self {
        theme::apply(&cc.egui_ctx);
        let mut cfg = Config::load();
        crate::i18n::init(cfg.language);
        // Presets de fábrica ainda com o nome original seguem o idioma ativo.
        cfg.relocalize_factory_names();
        if !Config::path().exists() {
            let _ = cfg.save();
        }
        let tools = ffmpeg::locate(&cfg.ffmpeg_dir);
        let queue = Queue::new(tools.clone());
        queue.set_repaint(cc.egui_ctx.clone());

        let (tx, rx) = mpsc::channel();
        let ctx = cc.egui_ctx.clone();
        ipc::serve(tx, move || ctx.request_repaint());

        let mut app = Self {
            current_preset: cfg.fallback_preset().map(|p| p.id.clone()).unwrap_or_default(),
            ext_text: cfg.extensions.join(", "),
            last_cfg: cfg.clone(),
            cfg,
            dirty_at: None,
            queue,
            tools,
            rx,
            tab: Tab::Queue,
            compact,
            edit_sel: 0,
            about: Default::default(),
            packaged: shell::is_packaged(),
            menu_installed: shell::is_installed(),
            modern_installed: shell::modern_installed(),
            toast: None,
            done_at: None,
        };
        if !initial.files.is_empty() || initial.tab.is_some() {
            app.handle_msg(&cc.egui_ctx, initial);
        }
        app
    }

    fn toast(&mut self, text: impl Into<String>, is_error: bool) {
        self.toast = Some((text.into(), is_error, Instant::now()));
    }

    fn expand(&mut self, ctx: &egui::Context) {
        if self.compact {
            self.compact = false;
            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(FULL_SIZE.into()));
        }
    }

    fn handle_msg(&mut self, ctx: &egui::Context, msg: Msg) {
        ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(false));
        ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
        if let Some(tab) = msg.tab.as_deref() {
            self.tab = match tab {
                "settings" => Tab::Settings,
                "about" => Tab::About,
                _ => Tab::Presets,
            };
            self.expand(ctx);
        }
        if msg.files.is_empty() {
            return;
        }
        let files = expand_paths(msg.files, &self.cfg.extensions);
        let wanted = msg.preset.as_deref().and_then(|id| self.cfg.preset(id)).cloned();
        match (wanted, msg.start) {
            (Some(preset), true) => self.queue.add(files, &preset, true),
            (_, start) => {
                // Sem preset (ou preset que não existe mais): só coloca na lista.
                if start {
                    self.toast(t!("toast-preset-missing"), true);
                }
                self.add_idle(files);
                self.expand(ctx);
            }
        }
        if msg.tab.is_none() {
            self.tab = Tab::Queue;
        }
    }

    fn add_idle(&mut self, files: Vec<PathBuf>) {
        let preset = self.cfg.preset(&self.current_preset).or(self.cfg.fallback_preset()).cloned();
        match preset {
            Some(p) => self.queue.add(files, &p, false),
            None => self.toast(t!("toast-create-preset-first"), true),
        }
    }

    /// Salva a configuração e propaga o que depende dela (menu contextual, ffmpeg).
    fn persist(&mut self) {
        if let Err(e) = self.cfg.save() {
            self.toast(t!("toast-save-failed", error => e), true);
        }
        if self.menu_installed && !self.packaged {
            if let Err(e) = shell::install(&self.cfg) {
                self.toast(t!("toast-menu-update-failed", error => e), true);
            }
        }
        if self.modern_installed && !self.packaged {
            // Só registra de novo se a lista de extensões mudou; os presets a DLL lê na hora.
            if let Err(e) = shell::install_modern(&self.cfg) {
                self.toast(t!("toast-modern-update-failed", error => e), true);
            }
        }
        self.dirty_at = None;
    }

    fn install_menus(&mut self) {
        let classic = shell::install(&self.cfg);
        let modern = if shell::modern_available() { Some(shell::install_modern(&self.cfg)) } else { None };
        match (classic, modern) {
            (Err(e), _) => self.toast(t!("toast-menu-install-failed", error => e), true),
            (Ok(()), Some(Err(e))) => self.toast(t!("toast-menu-modern-failed", error => e), true),
            (Ok(()), _) => self.toast(t!("toast-menu-installed"), false),
        }
        self.menu_installed = shell::is_installed();
        self.modern_installed = shell::modern_installed();
    }

    fn uninstall_menus(&mut self) {
        let classic = shell::uninstall().map_err(|e| e.to_string());
        let modern = shell::uninstall_modern();
        match classic.and(modern) {
            Ok(()) => self.toast(t!("toast-menu-removed"), false),
            Err(e) => self.toast(t!("toast-menu-remove-failed", error => e), true),
        }
        self.menu_installed = shell::is_installed();
        self.modern_installed = shell::modern_installed();
    }

    fn reload_tools(&mut self) {
        self.tools = ffmpeg::locate(&self.cfg.ffmpeg_dir);
        self.queue.set_tools(self.tools.clone());
    }

    fn tick(&mut self, ctx: &egui::Context) {
        while let Ok(msg) = self.rx.try_recv() {
            self.handle_msg(ctx, msg);
        }

        let dropped: Vec<PathBuf> = ctx.input(|i| i.raw.dropped_files.iter().filter_map(|f| f.path.clone()).collect());
        if !dropped.is_empty() {
            let files = expand_paths(dropped, &self.cfg.extensions);
            self.add_idle(files);
            self.expand(ctx);
            self.tab = Tab::Queue;
        }

        // Salvamento automático com um pequeno atraso depois da última edição.
        if self.cfg != self.last_cfg {
            if self.cfg.ffmpeg_dir != self.last_cfg.ffmpeg_dir {
                self.reload_tools();
            }
            self.last_cfg = self.cfg.clone();
            self.dirty_at = Some(Instant::now());
        }
        if let Some(at) = self.dirty_at {
            if at.elapsed() > Duration::from_millis(700) {
                self.persist();
            } else {
                ctx.request_repaint_after(Duration::from_millis(200));
            }
        }

        if let Some((_, _, at)) = &self.toast {
            if at.elapsed() > Duration::from_secs(5) {
                self.toast = None;
            } else {
                ctx.request_repaint_after(Duration::from_millis(500));
            }
        }

        // Janela rápida: fecha sozinha quando tudo terminou sem erro.
        if self.compact && self.cfg.close_when_done {
            let jobs = self.queue.snapshot();
            let busy = jobs.iter().any(|j| matches!(j.state, JobState::Idle | JobState::Queued | JobState::Running));
            let failed = jobs.iter().any(|j| j.state == JobState::Failed);
            if !jobs.is_empty() && !busy && !failed {
                let now = Instant::now();
                let at = self.done_at.get_or_insert(now);
                if ctx.input(|i| i.pointer.has_pointer()) {
                    *at = now; // o usuário está interagindo: espera
                }
                if at.elapsed() > Duration::from_millis(2500) {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                } else {
                    ctx.request_repaint_after(Duration::from_millis(250));
                }
            } else {
                self.done_at = None;
            }
        }
    }

    // ---------- Barra superior ----------

    fn top_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(RichText::new(crate::brand::APP_NAME).family(theme::semibold()).size(17.0));
            ui.add_space(14.0);
            for (tab, label) in [(Tab::Queue, t!("tab-queue")), (Tab::Presets, t!("tab-presets")), (Tab::Settings, t!("tab-settings")), (Tab::About, t!("tab-about"))] {
                let selected = self.tab == tab;
                let text = RichText::new(label).color(if selected { theme::TEXT } else { theme::MUTED });
                let fill = if selected { theme::CONTROL } else { Color32::TRANSPARENT };
                if ui.add(egui::Button::new(text).fill(fill)).clicked() {
                    self.tab = tab;
                }
            }
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if let Some((text, is_error, _)) = &self.toast {
                    let color = if *is_error { theme::ERR } else { theme::OK };
                    ui.add(egui::Label::new(RichText::new(text).size(12.5).color(color)).truncate());
                } else if self.tools.is_none() {
                    ui.label(RichText::new(t!("ffmpeg-missing-banner")).size(12.5).color(theme::ERR));
                }
            });
        });
    }

    // ---------- Fila ----------

    fn pick_files(&mut self) {
        let exts: Vec<&str> = self.cfg.extensions.iter().map(String::as_str).collect();
        let picked = rfd::FileDialog::new()
            .set_title(t!("dialog-add-videos"))
            .add_filter(t!("filter-videos"), &exts)
            .add_filter(t!("filter-all-files"), &["*"])
            .pick_files();
        if let Some(files) = picked {
            self.add_idle(files);
        }
    }

    fn queue_toolbar(&mut self, ui: &mut egui::Ui, jobs: &[Job]) {
        ui.horizontal(|ui| {
            if ui.button(t!("queue-add-files")).clicked() {
                self.pick_files();
            }
            ui.add_space(8.0);
            ui.label(RichText::new(t!("queue-preset")).color(theme::MUTED));
            let current = self.cfg.preset(&self.current_preset).map(|p| p.name.clone()).unwrap_or_else(|| "—".into());
            egui::ComboBox::from_id_salt("current_preset").width(190.0).selected_text(current).show_ui(ui, |ui| {
                for p in &self.cfg.presets {
                    ui.selectable_value(&mut self.current_preset, p.id.clone(), &p.name).on_hover_text(p.summary());
                }
            });
            let has_idle = jobs.iter().any(|j| j.state == JobState::Idle);
            if has_idle && ui.button(t!("queue-apply-all")).on_hover_text(t!("queue-apply-all-tip")).clicked() {
                if let Some(p) = self.cfg.preset(&self.current_preset) {
                    self.queue.set_preset_all_idle(p);
                }
            }
        });
    }

    fn job_list(&self, ui: &mut egui::Ui, jobs: &[Job]) {
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 6.0;
            for job in jobs {
                self.job_row(ui, job);
            }
        });
    }

    fn job_row(&self, ui: &mut egui::Ui, job: &Job) {
        let frame = egui::Frame::new()
            .fill(theme::CARD)
            .corner_radius(theme::radius(8))
            .inner_margin(egui::Margin::symmetric(12, 9));
        frame.show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.spacing_mut().item_spacing.y = 4.0;
            let size = egui::vec2(ui.available_width(), 40.0);
            ui.allocate_ui_with_layout(size, Layout::right_to_left(Align::Center), |ui| {
                self.job_actions(ui, job);
                ui.with_layout(Layout::top_down(Align::Min), |ui| {
                    ui.spacing_mut().item_spacing.y = 1.0;
                    let name = job.input.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
                    ui.add(egui::Label::new(RichText::new(name).family(theme::semibold())).truncate())
                        .on_hover_text(job.input.display().to_string());
                    let (text, color) = status_line(job);
                    let status = ui.add(egui::Label::new(RichText::new(text).size(12.5).color(color)).truncate());
                    if let Some(err) = &job.error {
                        status.on_hover_text(RichText::new(err).monospace().size(11.5));
                    }
                });
            });
            if job.state == JobState::Running {
                let bar = egui::ProgressBar::new(job.fraction.unwrap_or(0.0)).desired_height(4.0).fill(theme::ACCENT);
                ui.add(bar);
            }
        });
    }

    /// Botões da linha, adicionados da direita para a esquerda.
    fn job_actions(&self, ui: &mut egui::Ui, job: &Job) {
        let icon = |ui: &mut egui::Ui, glyph: &str, tip: &str| ui.button(glyph).on_hover_text(tip).clicked();
        match job.state {
            JobState::Idle => {
                if icon(ui, "✖", &t!("job-remove")) {
                    self.queue.remove(job.id);
                }
                egui::ComboBox::from_id_salt(("job_preset", job.id))
                    .width(160.0)
                    .selected_text(&job.preset.name)
                    .show_ui(ui, |ui| {
                        for p in &self.cfg.presets {
                            if ui.selectable_label(p.id == job.preset.id, &p.name).on_hover_text(p.summary()).clicked() {
                                self.queue.set_preset(job.id, p);
                            }
                        }
                    });
            }
            JobState::Queued | JobState::Running => {
                if icon(ui, "✖", &t!("job-cancel")) {
                    self.queue.cancel(job.id);
                }
            }
            JobState::Done => {
                if icon(ui, "✖", &t!("job-remove")) {
                    self.queue.remove(job.id);
                }
                if icon(ui, &t!("job-open-folder"), &t!("job-open-folder-tip")) {
                    if let Some(out) = &job.output {
                        reveal(out);
                    }
                }
            }
            JobState::Failed | JobState::Cancelled => {
                if icon(ui, "✖", &t!("job-remove")) {
                    self.queue.remove(job.id);
                }
                if icon(ui, "⟲", &t!("job-retry")) {
                    self.queue.start(job.id);
                }
            }
        }
    }

    fn queue_footer(&mut self, ui: &mut egui::Ui, jobs: &[Job]) {
        let count = |s: JobState| jobs.iter().filter(|j| j.state == s).count();
        let idle = count(JobState::Idle);
        let active = count(JobState::Queued) + count(JobState::Running);
        let finished = count(JobState::Done) + count(JobState::Failed) + count(JobState::Cancelled);
        ui.horizontal(|ui| {
            let mut parts = vec![t!("queue-count", count => jobs.len())];
            if active > 0 {
                parts.push(t!("queue-active", count => active));
            }
            if count(JobState::Done) > 0 {
                parts.push(t!("queue-done", count => count(JobState::Done)));
            }
            if count(JobState::Failed) > 0 {
                parts.push(t!("queue-failed", count => count(JobState::Failed)));
            }
            ui.label(RichText::new(parts.join("  ·  ")).color(theme::MUTED).size(12.5));

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if idle > 0 {
                    let label = if idle == 1 { t!("queue-encode-one") } else { t!("queue-encode-many", count => idle) };
                    if primary_button(ui, &label).clicked() {
                        self.queue.start_all_idle();
                    }
                }
                if active > 0 && ui.button(t!("queue-cancel-all")).clicked() {
                    self.queue.cancel_all();
                }
                if finished > 0 && ui.button(t!("queue-clear-finished")).clicked() {
                    self.queue.clear_finished();
                }
            });
        });
    }

    fn empty_state(&self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space((ui.available_height() * 0.32).max(20.0));
            ui.label(RichText::new(t!("queue-empty-title")).family(theme::semibold()).size(19.0));
            ui.add_space(2.0);
            ui.label(
                RichText::new(t!("queue-empty-hint", menu => crate::brand::MENU_NAME)).color(theme::MUTED),
            );
        });
    }

    fn queue_tab(&mut self, ui: &mut egui::Ui, jobs: &[Job]) {
        egui::Panel::bottom("queue_footer").frame(bar_frame()).show(ui, |ui| self.queue_footer(ui, jobs));
        egui::CentralPanel::default().frame(page_frame()).show(ui, |ui| {
            self.queue_toolbar(ui, jobs);
            ui.add_space(6.0);
            if jobs.is_empty() {
                self.empty_state(ui);
            } else {
                self.job_list(ui, jobs);
            }
        });
    }

    fn compact_ui(&mut self, ui: &mut egui::Ui, jobs: &[Job]) {
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(theme::BG).inner_margin(egui::Margin::same(12)))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(crate::brand::APP_NAME).family(theme::semibold()).size(15.0));
                    if let Some((text, is_error, _)) = &self.toast {
                        ui.label(RichText::new(text).size(12.5).color(if *is_error { theme::ERR } else { theme::OK }));
                    } else if self.done_at.is_some() {
                        ui.label(RichText::new(t!("compact-done")).size(12.5).color(theme::OK));
                    }
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui.button(t!("compact-open-full")).clicked() {
                            let ctx = ui.ctx().clone();
                            self.expand(&ctx);
                        }
                    });
                });
                ui.add_space(4.0);
                self.job_list(ui, jobs);
            });
    }

    // ---------- Presets ----------

    fn presets_tab(&mut self, ui: &mut egui::Ui) {
        self.edit_sel = self.edit_sel.min(self.cfg.presets.len().saturating_sub(1));

        egui::Panel::left("preset_list")
            .exact_size(250.0)
            .resizable(false)
            .frame(egui::Frame::new().fill(theme::PANEL).inner_margin(egui::Margin::same(12)))
            .show(ui, |ui| self.preset_list(ui));

        egui::CentralPanel::default().frame(page_frame()).show(ui, |ui| {
            let encoders = self.tools.as_ref().map(|t| t.encoders.clone()).unwrap_or_default();
            let sel = self.edit_sel;
            match self.cfg.presets.get_mut(sel) {
                Some(preset) => {
                    egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                        preset_editor(ui, preset, &encoders);
                    });
                }
                None => {
                    ui.label(RichText::new(t!("presets-empty")).color(theme::MUTED));
                }
            }
        });
    }

    fn preset_list(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button(t!("presets-new")).clicked() {
                let mut p = Preset::default();
                p.id = self.cfg.unique_id(&p.name);
                self.cfg.presets.push(p);
                self.edit_sel = self.cfg.presets.len() - 1;
            }
            let has_sel = self.edit_sel < self.cfg.presets.len();
            if ui.add_enabled(has_sel, egui::Button::new(t!("presets-duplicate"))).clicked() {
                let mut p = self.cfg.presets[self.edit_sel].clone();
                p.name = t!("presets-copy-name", name => p.name);
                p.id = self.cfg.unique_id(&p.name);
                self.cfg.presets.insert(self.edit_sel + 1, p);
                self.edit_sel += 1;
            }
        });
        ui.add_space(4.0);

        let mut clicked = None;
        egui::ScrollArea::vertical().auto_shrink([false, false]).max_height(ui.available_height() - 44.0).show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 2.0;
            for (i, p) in self.cfg.presets.iter().enumerate() {
                let selected = i == self.edit_sel;
                let frame = egui::Frame::new()
                    .fill(if selected { theme::CONTROL } else { Color32::TRANSPARENT })
                    .corner_radius(theme::radius(6))
                    .inner_margin(egui::Margin::symmetric(10, 7));
                let resp = frame
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.spacing_mut().item_spacing.y = 0.0;
                        let name = RichText::new(&p.name).family(theme::semibold());
                        ui.add(egui::Label::new(name).selectable(false).truncate());
                        let mut summary = p.summary();
                        if !p.in_menu {
                            summary.push_str(&format!(" · {}", t!("presets-not-in-menu")));
                        }
                        ui.add(
                            egui::Label::new(RichText::new(summary).size(12.0).color(theme::MUTED))
                                .selectable(false)
                                .truncate(),
                        );
                    })
                    .response
                    .interact(egui::Sense::click());
                if resp.clicked() {
                    clicked = Some(i);
                }
            }
        });
        if let Some(i) = clicked {
            self.edit_sel = i;
        }

        ui.add_space(4.0);
        ui.horizontal(|ui| {
            let n = self.cfg.presets.len();
            let sel = self.edit_sel;
            if ui.add_enabled(sel > 0 && n > 1, egui::Button::new("↑")).on_hover_text(t!("presets-move-up")).clicked() {
                self.cfg.presets.swap(sel, sel - 1);
                self.edit_sel -= 1;
            }
            if ui.add_enabled(sel + 1 < n, egui::Button::new("↓")).on_hover_text(t!("presets-move-down")).clicked() {
                self.cfg.presets.swap(sel, sel + 1);
                self.edit_sel += 1;
            }
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui.add_enabled(n > 1, egui::Button::new(t!("presets-delete"))).clicked() {
                    self.cfg.presets.remove(sel);
                    self.edit_sel = sel.min(self.cfg.presets.len() - 1);
                }
            });
        });
    }

    // ---------- Ajustes ----------

    fn settings_tab(&mut self, ui: &mut egui::Ui) {
        egui::CentralPanel::default().frame(page_frame()).show(ui, |ui| {
            egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                ui.spacing_mut().item_spacing.y = 10.0;

                if self.packaged {
                    section(ui, &t!("settings-menu"), |ui| {
                        ui.label(RichText::new(t!("settings-menu-store")).color(theme::OK));
                        hint(ui, &t!("settings-menu-store-hint", menu => crate::brand::MENU_NAME));
                    });
                }
                if !self.packaged {
                section(ui, &t!("settings-menu"), |ui| {
                    ui.horizontal(|ui| {
                        let installed = self.menu_installed || self.modern_installed;
                        let (text, color) = match (self.modern_installed, self.menu_installed) {
                            (true, true) => (t!("settings-menu-both"), theme::OK),
                            (true, false) => (t!("settings-menu-modern-only"), theme::OK),
                            (false, true) => (t!("settings-menu-classic-only"), theme::OK),
                            (false, false) => (t!("settings-menu-not-installed"), theme::MUTED),
                        };
                        ui.label(RichText::new(text).color(color));
                        ui.add_space(8.0);
                        if primary_button(ui, &if installed { t!("settings-update") } else { t!("settings-install") }).clicked() {
                            self.install_menus();
                        }
                        if installed && ui.button(t!("settings-remove")).clicked() {
                            self.uninstall_menus();
                        }
                    });
                    hint(ui, &t!("settings-menu-hint", menu => crate::brand::MENU_NAME));
                    if self.menu_installed && !self.modern_installed {
                        let why = if shell::modern_available() {
                            t!("settings-modern-needs-devmode")
                        } else {
                            t!("settings-modern-unavailable")
                        };
                        hint(ui, &why);
                    }
                });
                }

                let ext_title = if self.packaged { t!("settings-extensions-store") } else { t!("settings-extensions") };
                let ext_hint = if self.packaged {
                    t!("settings-extensions-store-hint")
                } else {
                    t!("settings-extensions-hint")
                };
                section(ui, &ext_title, |ui| {
                    let edit = egui::TextEdit::multiline(&mut self.ext_text).desired_rows(2).desired_width(f32::INFINITY);
                    if ui.add(edit).changed() {
                        self.cfg.extensions = self
                            .ext_text
                            .split([',', ' ', ';', '\n'])
                            .map(|e| e.trim().trim_start_matches('.').to_lowercase())
                            .filter(|e| !e.is_empty())
                            .collect();
                    }
                    hint(ui, &ext_hint);
                });

                section(ui, &t!("settings-ffmpeg"), |ui| {
                    match &self.tools {
                        Some(t) => {
                            ui.label(RichText::new(t!("settings-ffmpeg-found", path => t.ffmpeg.display())).color(theme::OK));
                        }
                        None => {
                            ui.label(RichText::new(t!("settings-ffmpeg-missing")).color(theme::ERR));
                        }
                    }
                    ui.horizontal(|ui| {
                        if ui.button(t!("browse")).clicked() {
                            if let Some(dir) = rfd::FileDialog::new().set_title(t!("settings-ffmpeg-folder")).pick_folder() {
                                self.cfg.ffmpeg_dir = dir.display().to_string();
                            }
                        }
                        ui.add(
                            egui::TextEdit::singleline(&mut self.cfg.ffmpeg_dir)
                                .hint_text(t!("settings-ffmpeg-auto"))
                                .desired_width(f32::INFINITY),
                        );
                    });
                });

                section(ui, &t!("settings-language"), |ui| {
                    let before = self.cfg.language;
                    egui::ComboBox::from_id_salt("language").width(220.0).selected_text(self.cfg.language.label()).show_ui(ui, |ui| {
                        for lang in Language::ALL {
                            ui.selectable_value(&mut self.cfg.language, lang, lang.label());
                        }
                    });
                    if self.cfg.language != before {
                        // Troca na hora; o menu de botão direito acompanha ao salvar.
                        crate::i18n::init(self.cfg.language);
                        self.cfg.relocalize_factory_names();
                    }
                });

                section(ui, &t!("settings-behavior"), |ui| {
                    ui.checkbox(&mut self.cfg.close_when_done, t!("settings-close-when-done"));
                    ui.horizontal(|ui| {
                        ui.label(t!("settings-default-preset"));
                        let name = self.cfg.fallback_preset().map(|p| p.name.clone()).unwrap_or_default();
                        egui::ComboBox::from_id_salt("default_preset").width(190.0).selected_text(name).show_ui(ui, |ui| {
                            for p in &self.cfg.presets {
                                ui.selectable_value(&mut self.cfg.default_preset, p.id.clone(), &p.name);
                            }
                        });
                    });
                });

                hint(ui, &t!("settings-config-path", path => Config::path().display()));
            });
        });
    }
}

impl eframe::App for App {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.tick(ctx);
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let jobs = self.queue.snapshot();
        let screen = ui.max_rect();

        if self.compact {
            self.compact_ui(ui, &jobs);
        } else {
            egui::Panel::top("top_bar").frame(bar_frame()).show(ui, |ui| self.top_bar(ui));
            match self.tab {
                Tab::Queue => self.queue_tab(ui, &jobs),
                Tab::Presets => self.presets_tab(ui),
                Tab::Settings => self.settings_tab(ui),
                Tab::About => {
                    egui::CentralPanel::default().frame(page_frame()).show(ui, |ui| {
                        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                            ui.set_max_width(640.0);
                            self.about.ui(ui);
                        });
                    });
                }
            }
        }

        // Destaque enquanto arquivos são arrastados sobre a janela.
        if ui.ctx().input(|i| !i.raw.hovered_files.is_empty()) {
            let layer = egui::LayerId::new(egui::Order::Foreground, egui::Id::new("drop_overlay"));
            let painter = ui.ctx().layer_painter(layer);
            painter.rect_filled(screen, 0.0, Color32::from_black_alpha(190));
            painter.rect_stroke(
                screen.shrink(10.0),
                theme::radius(12),
                egui::Stroke::new(2.0, theme::ACCENT),
                egui::StrokeKind::Inside,
            );
            painter.text(
                screen.center(),
                egui::Align2::CENTER_CENTER,
                t!("drop-overlay"),
                egui::FontId::new(20.0, theme::semibold()),
                theme::TEXT,
            );
        }
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        if self.dirty_at.is_some() {
            self.persist();
        }
        // Não deixa ffmpeg órfão: cancela e espera a limpeza dos arquivos parciais.
        self.queue.cancel_all();
        let deadline = Instant::now() + Duration::from_millis(1500);
        while Instant::now() < deadline && self.queue.snapshot().iter().any(|j| j.state == JobState::Running) {
            std::thread::sleep(Duration::from_millis(30));
        }
    }
}

// ---------- Editor de preset ----------

fn row(ui: &mut egui::Ui, label: &str, add: impl FnOnce(&mut egui::Ui)) {
    ui.label(RichText::new(label).color(theme::MUTED));
    ui.horizontal(add);
    ui.end_row();
}

fn form(ui: &mut egui::Ui, id: &str, title: &str, add: impl FnOnce(&mut egui::Ui)) {
    section(ui, title, |ui| {
        egui::Grid::new(id).num_columns(2).min_col_width(130.0).spacing([14.0, 9.0]).show(ui, add);
    });
}

fn enum_combo<T: Copy + PartialEq>(ui: &mut egui::Ui, id: &str, value: &mut T, options: &[T], label: fn(T) -> String) {
    egui::ComboBox::from_id_salt(id).width(250.0).selected_text(label(*value)).show_ui(ui, |ui| {
        for &option in options {
            ui.selectable_value(value, option, label(option));
        }
    });
}

fn num_combo(ui: &mut egui::Ui, id: &str, value: &mut u32, options: &[(u32, &str)]) {
    // "Original" é o único rótulo que muda com o idioma.
    let show = |label: &str| if label == "Original" { t!("original") } else { label.to_string() };
    let current = options.iter().find(|(v, _)| v == value).map(|(_, l)| show(l)).unwrap_or_else(|| value.to_string());
    egui::ComboBox::from_id_salt(id).width(250.0).selected_text(current).show_ui(ui, |ui| {
        for (v, label) in options {
            ui.selectable_value(value, *v, show(label));
        }
    });
}

const RESOLUTIONS: [(u32, &str); 7] = [
    (0, "Original"),
    (2160, "2160p (4K)"),
    (1440, "1440p"),
    (1080, "1080p"),
    (720, "720p"),
    (480, "480p"),
    (360, "360p"),
];
const FRAME_RATES: [(u32, &str); 8] =
    [(0, "Original"), (60, "60 fps"), (30, "30 fps"), (25, "25 fps"), (24, "24 fps"), (15, "15 fps"), (12, "12 fps"), (10, "10 fps")];

fn preset_editor(ui: &mut egui::Ui, p: &mut Preset, encoders: &HashSet<String>) {
    ui.spacing_mut().item_spacing.y = 10.0;

    form(ui, "f_general", &t!("editor-general"), |ui| {
        row(ui, &t!("editor-name"), |ui| {
            ui.add(egui::TextEdit::singleline(&mut p.name).desired_width(262.0));
        });
        row(ui, &t!("editor-format"), |ui| {
            let before = p.container;
            enum_combo(ui, "container", &mut p.container, &Container::ALL, Container::label);
            if p.container != before {
                p.normalize();
            }
        });
        row(ui, &t!("editor-context-menu"), |ui| {
            ui.checkbox(&mut p.in_menu, t!("editor-show-in-menu"));
        });
    });

    if p.container == Container::Gif {
        form(ui, "f_gif", &t!("editor-gif"), |ui| {
            row(ui, &t!("editor-max-size"), |ui| num_combo(ui, "res", &mut p.max_short_side, &RESOLUTIONS[..]));
            row(ui, &t!("editor-max-fps"), |ui| num_combo(ui, "fps", &mut p.max_fps, &FRAME_RATES[..]));
        });
    } else if p.has_video_settings() {
        form(ui, "f_video", &t!("editor-video"), |ui| {
            row(ui, &t!("editor-codec"), |ui| enum_combo(ui, "vcodec", &mut p.video, p.container.video_codecs(), VideoCodec::label));
            if p.video == VideoCodec::Copy {
                return;
            }
            let uses_hw = !matches!(p.video, VideoCodec::Vp9 | VideoCodec::Prores);
            if uses_hw {
                row(ui, &t!("editor-encoder"), |ui| enum_combo(ui, "hw", &mut p.hw, &Hw::ALL, Hw::label));
            }
            if p.video != VideoCodec::Prores {
                row(ui, &t!("editor-rate-control"), |ui| enum_combo(ui, "rate", &mut p.rate_mode, &RateMode::ALL, RateMode::label));
                match p.rate_mode {
                    RateMode::Quality => row(ui, &t!("editor-quality"), |ui| {
                        ui.add(egui::DragValue::new(&mut p.crf).range(0..=63).speed(0.2));
                        hint(ui, &t!("editor-quality-hint"));
                    }),
                    RateMode::Bitrate => row(ui, &t!("editor-video-bitrate"), |ui| {
                        ui.add(egui::DragValue::new(&mut p.video_kbps).range(100..=200_000).speed(25.0).suffix(" kbps"));
                        hint(ui, &crate::config::fmt_kbps(p.video_kbps));
                    }),
                    RateMode::TargetSize => row(ui, &t!("editor-file-size"), |ui| {
                        ui.add(egui::DragValue::new(&mut p.target_mb).range(1..=100_000).speed(0.5).suffix(" MB"));
                        hint(ui, &t!("editor-file-size-hint"));
                    }),
                }
                row(ui, &t!("editor-speed"), |ui| enum_combo(ui, "speed", &mut p.speed, &Speed::ALL, Speed::label));
            }
            row(ui, &t!("editor-max-resolution"), |ui| num_combo(ui, "res", &mut p.max_short_side, &RESOLUTIONS[..]));
            row(ui, &t!("editor-max-fps"), |ui| num_combo(ui, "fps", &mut p.max_fps, &FRAME_RATES[..]));
        });
    }

    if p.container != Container::Gif {
        form(ui, "f_audio", &t!("editor-audio"), |ui| {
            row(ui, &t!("editor-codec"), |ui| enum_combo(ui, "acodec", &mut p.audio, p.container.audio_codecs(), AudioCodec::label));
            if matches!(p.audio, AudioCodec::Aac | AudioCodec::Opus | AudioCodec::Mp3) {
                row(ui, &t!("editor-bitrate"), |ui| {
                    ui.add(egui::DragValue::new(&mut p.audio_kbps).range(16..=512).speed(1.0).suffix(" kbps"));
                });
            }
        });
    }

    form(ui, "f_output", &t!("editor-output"), |ui| {
        row(ui, &t!("editor-suffix"), |ui| {
            ui.add(egui::TextEdit::singleline(&mut p.suffix).desired_width(262.0));
        });
        row(ui, &t!("editor-folder"), |ui| {
            ui.add(
                egui::TextEdit::singleline(&mut p.output_dir)
                    .hint_text(t!("editor-folder-hint"))
                    .desired_width(262.0),
            );
            if ui.button(t!("browse")).clicked() {
                if let Some(dir) = rfd::FileDialog::new().pick_folder() {
                    p.output_dir = dir.display().to_string();
                }
            }
        });
        row(ui, "", |ui| {
            hint(ui, &t!("editor-output-example", suffix => p.suffix.trim(), ext => p.container.ext()));
        });
        row(ui, &t!("editor-extra-args"), |ui| {
            ui.add(egui::TextEdit::singleline(&mut p.extra_args).hint_text(t!("editor-optional")).desired_width(262.0));
        });
    });

    section(ui, &t!("editor-command"), |ui| {
        let cmd = ffmpeg::preview(p, encoders);
        ui.add(egui::Label::new(RichText::new(cmd).monospace().size(12.0).color(theme::MUTED)).wrap());
    });
}

// ---------- Auxiliares ----------

fn page_frame() -> egui::Frame {
    egui::Frame::new().fill(theme::BG).inner_margin(egui::Margin::same(16))
}

fn bar_frame() -> egui::Frame {
    egui::Frame::new().fill(theme::PANEL).inner_margin(egui::Margin::symmetric(16, 10))
}

pub(crate) fn primary_button(ui: &mut egui::Ui, label: &str) -> egui::Response {
    let visuals = &mut ui.visuals_mut().widgets;
    let saved = (visuals.hovered.weak_bg_fill, visuals.active.weak_bg_fill);
    visuals.hovered.weak_bg_fill = theme::ACCENT_HOVER;
    visuals.active.weak_bg_fill = theme::ACCENT_HOVER;
    let text = RichText::new(label).family(theme::semibold()).color(theme::ON_ACCENT);
    let resp = ui.add(egui::Button::new(text).fill(theme::ACCENT));
    let visuals = &mut ui.visuals_mut().widgets;
    (visuals.hovered.weak_bg_fill, visuals.active.weak_bg_fill) = saved;
    resp
}

fn section(ui: &mut egui::Ui, title: &str, add: impl FnOnce(&mut egui::Ui)) {
    egui::Frame::new()
        .fill(theme::CARD)
        .corner_radius(theme::radius(8))
        .inner_margin(egui::Margin::symmetric(14, 12))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(RichText::new(title).family(theme::semibold()).size(13.0).color(theme::MUTED));
            ui.add_space(2.0);
            add(ui);
        });
}

fn hint(ui: &mut egui::Ui, text: &str) {
    ui.add(egui::Label::new(RichText::new(text).size(12.5).color(theme::MUTED)).wrap());
}

fn status_line(job: &Job) -> (String, Color32) {
    let preset = &job.preset.name;
    match job.state {
        JobState::Idle => (t!("status-ready", size => fmt_size(job.in_bytes)), theme::MUTED),
        JobState::Queued => (t!("status-queued", preset => preset), theme::MUTED),
        JobState::Running => {
            let mut text = match job.fraction {
                Some(f) => {
                    let mut t = t!("status-progress", preset => preset, percent => format!("{:.0}", f * 100.0));
                    if job.speed > 0.0 {
                        t.push_str(&format!(" · {}", t!("status-speed", speed => format!("{:.1}", job.speed))));
                    }
                    if let Some(eta) = job.eta_secs {
                        t.push_str(&format!(" · {}", t!("status-eta", time => fmt_time(eta))));
                    }
                    t
                }
                None => t!("status-encoding", preset => preset),
            };
            if let Some(note) = &job.note {
                text.push_str(&format!(" · {note}"));
            }
            (text, theme::TEXT)
        }
        JobState::Done => {
            let mut text = t!("status-done", preset => preset, from => fmt_size(job.in_bytes), to => fmt_size(job.out_bytes));
            if job.in_bytes > 0 && job.out_bytes > 0 {
                let delta = (job.out_bytes as f64 / job.in_bytes as f64 - 1.0) * 100.0;
                text.push_str(&format!(" ({}{:.0}%)", if delta > 0.0 { "+" } else { "−" }, delta.abs()));
            }
            (text, theme::OK)
        }
        JobState::Failed => {
            let last = job.error.as_deref().and_then(|e| e.lines().last()).map(String::from).unwrap_or_else(|| t!("status-unknown-error"));
            (t!("status-error", message => last), theme::ERR)
        }
        JobState::Cancelled => (t!("status-cancelled", preset => preset), theme::MUTED),
    }
}

fn fmt_size(bytes: u64) -> String {
    let mb = bytes as f64 / 1_048_576.0;
    if mb >= 1024.0 {
        format!("{:.2} GB", mb / 1024.0)
    } else if mb >= 10.0 {
        format!("{mb:.0} MB")
    } else if mb >= 0.1 {
        format!("{mb:.1} MB")
    } else {
        format!("{:.0} KB", bytes as f64 / 1024.0)
    }
}

fn fmt_time(secs: f32) -> String {
    let s = secs.max(0.0).round() as u64;
    if s >= 3600 {
        format!("{}:{:02}:{:02}", s / 3600, s % 3600 / 60, s % 60)
    } else {
        format!("{}:{:02}", s / 60, s % 60)
    }
}

/// Abre o Explorer com o arquivo selecionado.
fn reveal(path: &Path) {
    let _ = std::process::Command::new("explorer.exe").raw_arg(format!("/select,\"{}\"", path.display())).spawn();
}

/// Troca pastas pelos vídeos dentro delas (pelas extensões configuradas).
fn expand_paths(paths: Vec<PathBuf>, exts: &[String]) -> Vec<PathBuf> {
    fn walk(dir: &Path, exts: &[String], depth: u32, out: &mut Vec<PathBuf>) {
        let Ok(entries) = std::fs::read_dir(dir) else { return };
        let mut entries: Vec<PathBuf> = entries.filter_map(|e| e.ok().map(|e| e.path())).collect();
        entries.sort();
        for path in entries {
            if path.is_dir() {
                if depth < 6 {
                    walk(&path, exts, depth + 1, out);
                }
            } else if path
                .extension()
                .map(|e| e.to_string_lossy().to_lowercase())
                .is_some_and(|e| exts.iter().any(|x| *x == e))
            {
                out.push(path);
            }
        }
    }
    let mut out = Vec::new();
    for path in paths {
        if path.is_dir() {
            walk(&path, exts, 0, &mut out);
        } else {
            out.push(path);
        }
    }
    out
}
