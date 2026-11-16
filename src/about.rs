//! Aba "Sobre": identidade, dedicatória, autor, doação e licenças — no mesmo
//! formato do Sobre do MariPDF.

use crate::brand::APP_NAME;
use crate::shell::{is_packaged, open_url};
use crate::theme;
use eframe::egui::{self, RichText};

const REPO_URL: &str = "https://github.com/flavioflavioflavio/QuickEncoder";
const RELEASES_URL: &str = "https://github.com/flavioflavioflavio/QuickEncoder/releases";
const LINKEDIN_URL: &str = "https://www.linkedin.com/in/takemotoflavio/";
const EVOMIND_URL: &str = "https://evomind.app/investors";
/// Mesmo canal oficial de doação do MariPDF (botão hospedado do PayPal do autor).
const DONATION_URL: &str = "https://www.paypal.com/donate/?hosted_button_id=VF4HWVKJTLJ7E";
const FFMPEG_URL: &str = "https://ffmpeg.org/";
const FFMPEG_SOURCE_URL: &str = "https://github.com/BtbN/FFmpeg-Builds/releases/tag/autobuild-2026-11-15-12-40";

const LOGO_HEIGHT: f32 = 34.0;

#[derive(Default)]
pub struct About {
    logo: Option<egui::TextureHandle>,
}

fn section_title(ui: &mut egui::Ui, text: &str) {
    ui.label(RichText::new(text).family(theme::semibold()).size(12.0).color(theme::MUTED));
}

fn link_row(ui: &mut egui::Ui, links: &[(&str, &str)]) {
    ui.horizontal_wrapped(|ui| {
        for (index, (label, url)) in links.iter().enumerate() {
            if index > 0 {
                ui.label(RichText::new("·").color(theme::MUTED));
            }
            let link = ui.link(RichText::new(*label).size(13.0).color(theme::ACCENT));
            if link.on_hover_text(*url).clicked() {
                open_url(url);
            }
        }
    });
}

impl About {
    fn logo(&mut self, ctx: &egui::Context) -> Option<egui::TextureHandle> {
        if self.logo.is_none() {
            let icon = eframe::icon_data::from_png_bytes(include_bytes!("../assets/logo-horizontal.png")).ok()?;
            let image = egui::ColorImage::from_rgba_unmultiplied([icon.width as usize, icon.height as usize], &icon.rgba);
            self.logo = Some(ctx.load_texture("about-logo", image, egui::TextureOptions::LINEAR));
        }
        self.logo.clone()
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.spacing_mut().item_spacing.y = 6.0;
        let version = env!("CARGO_PKG_VERSION");

        // ── Identidade ──────────────────────────────────────────────────────
        match self.logo(ui.ctx()) {
            Some(tex) => {
                let size = tex.size_vec2();
                let scale = LOGO_HEIGHT / size.y;
                ui.add(egui::Image::new((tex.id(), size * scale)));
            }
            None => {
                ui.label(RichText::new(APP_NAME).family(theme::semibold()).size(22.0));
            }
        }
        ui.label(RichText::new(t!("about-version", version => version)).size(12.5).color(theme::MUTED));
        ui.add_space(2.0);
        ui.label(
            RichText::new(t!("about-desc")).size(14.0).color(theme::TEXT),
        );

        // ── Dedicatória ─────────────────────────────────────────────────────
        ui.add_space(6.0);
        egui::Frame::new()
            .fill(theme::ACCENT.gamma_multiply(0.14))
            .corner_radius(theme::radius(10))
            .inner_margin(egui::Margin::symmetric(14, 10))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.horizontal(|ui| {
                    ui.label(RichText::new("♥").size(15.0).color(theme::ACCENT));
                    ui.add_space(4.0);
                    ui.label(RichText::new(t!("about-dedication")).size(14.0).italics().color(theme::TEXT));
                });
            });

        // ── Autor ───────────────────────────────────────────────────────────
        ui.add_space(10.0);
        section_title(ui, &t!("about-author-title"));
        ui.label(RichText::new("Flávio Takemoto").family(theme::semibold()).size(16.0).color(theme::TEXT));
        ui.add(
            egui::Label::new(
                RichText::new(t!("about-author-bio"))
                .size(12.5)
                .color(theme::MUTED),
            )
            .wrap(),
        );
        link_row(ui, &[("LinkedIn", LINKEDIN_URL), ("EvoMind", EVOMIND_URL), (&t!("about-link-github"), REPO_URL)]);

        // ── Apoio ───────────────────────────────────────────────────────────
        ui.add_space(10.0);
        section_title(ui, &t!("about-support-title"));
        ui.add(
            egui::Label::new(
                RichText::new(t!("about-support-desc", app => APP_NAME))
                .size(12.5)
                .color(theme::MUTED),
            )
            .wrap(),
        );
        ui.add_space(2.0);
        if crate::app::primary_button(ui, &format!("  {}  ", t!("about-donate"))).on_hover_text(DONATION_URL).clicked() {
            open_url(DONATION_URL);
        }

        // ── Atualizações ────────────────────────────────────────────────────
        ui.add_space(10.0);
        section_title(ui, &t!("about-updates-title"));
        if is_packaged() {
            ui.label(RichText::new(t!("about-updates-store")).size(12.5).color(theme::MUTED));
        } else {
            link_row(ui, &[(&t!("about-updates-github"), RELEASES_URL)]);
        }

        // ── Licenças ────────────────────────────────────────────────────────
        ui.add_space(10.0);
        section_title(ui, &t!("about-licenses-title"));
        ui.add(
            egui::Label::new(
                RichText::new(t!("about-licenses-desc"))
                .size(12.0)
                .color(theme::MUTED),
            )
            .wrap(),
        );
        link_row(ui, &[("FFmpeg", FFMPEG_URL), (&t!("about-ffmpeg-source"), FFMPEG_SOURCE_URL)]);
        ui.add_space(6.0);
        ui.label(RichText::new(t!("about-license")).size(11.5).color(theme::MUTED));
    }
}
