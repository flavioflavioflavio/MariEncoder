//! Tema escuro do app: cores, fontes do sistema e espaçamentos.

use eframe::egui::{self, Color32, CornerRadius, FontFamily, FontId, Stroke, TextStyle};
use std::sync::Arc;

pub const BG: Color32 = Color32::from_rgb(0x12, 0x12, 0x15);
pub const PANEL: Color32 = Color32::from_rgb(0x18, 0x18, 0x1c);
pub const CARD: Color32 = Color32::from_rgb(0x20, 0x20, 0x26);
pub const CONTROL: Color32 = Color32::from_rgb(0x2b, 0x2b, 0x33);
pub const CONTROL_HOVER: Color32 = Color32::from_rgb(0x36, 0x36, 0x40);
pub const BORDER: Color32 = Color32::from_rgb(0x30, 0x30, 0x39);
pub const TEXT: Color32 = Color32::from_rgb(0xec, 0xec, 0xf1);
pub const MUTED: Color32 = Color32::from_rgb(0x8f, 0x8f, 0x9c);
pub const ACCENT: Color32 = Color32::from_rgb(0xff, 0x6b, 0x2c);
pub const ACCENT_HOVER: Color32 = Color32::from_rgb(0xff, 0x82, 0x4a);
pub const ON_ACCENT: Color32 = Color32::from_rgb(0x1a, 0x0d, 0x06);
pub const OK: Color32 = Color32::from_rgb(0x3d, 0xd6, 0x8c);
pub const ERR: Color32 = Color32::from_rgb(0xff, 0x62, 0x62);

pub fn radius(r: u8) -> CornerRadius {
    CornerRadius::same(r)
}

pub fn semibold() -> FontFamily {
    FontFamily::Name("semibold".into())
}

pub fn apply(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    let fallback = fonts.families.get(&FontFamily::Proportional).cloned().unwrap_or_default();
    let windir = std::env::var("WINDIR").unwrap_or_else(|_| r"C:\Windows".into());
    let mut load = |key: &str, file: &str| -> bool {
        match std::fs::read(format!(r"{windir}\Fonts\{file}")) {
            Ok(bytes) => {
                fonts.font_data.insert(key.into(), Arc::new(egui::FontData::from_owned(bytes)));
                true
            }
            Err(_) => false,
        }
    };
    let has_regular = load("segoe", "segoeui.ttf");
    let has_semibold = load("segoe-semibold", "seguisb.ttf");

    let mut regular = fallback.clone();
    if has_regular {
        regular.insert(0, "segoe".into());
    }
    let mut semi = regular.clone();
    if has_semibold {
        semi.insert(0, "segoe-semibold".into());
    }
    fonts.families.insert(FontFamily::Proportional, regular);
    fonts.families.insert(semibold(), semi);
    ctx.set_fonts(fonts);

    ctx.set_theme(egui::ThemePreference::Dark);
    ctx.all_styles_mut(|s| {
        s.text_styles = [
            (TextStyle::Heading, FontId::new(20.0, semibold())),
            (TextStyle::Body, FontId::new(14.5, FontFamily::Proportional)),
            (TextStyle::Button, FontId::new(14.5, FontFamily::Proportional)),
            (TextStyle::Small, FontId::new(12.5, FontFamily::Proportional)),
            (TextStyle::Monospace, FontId::new(12.5, FontFamily::Monospace)),
        ]
        .into();
        s.spacing.item_spacing = egui::vec2(8.0, 8.0);
        s.spacing.button_padding = egui::vec2(12.0, 6.0);
        s.spacing.interact_size.y = 30.0;
        s.spacing.combo_width = 150.0;

        let v = &mut s.visuals;
        *v = egui::Visuals::dark();
        v.panel_fill = BG;
        v.window_fill = PANEL;
        v.window_stroke = Stroke::new(1.0, BORDER);
        v.extreme_bg_color = Color32::from_rgb(0x0e, 0x0e, 0x11);
        v.faint_bg_color = CARD;
        v.hyperlink_color = ACCENT;
        v.selection.bg_fill = ACCENT.gamma_multiply(0.35);
        v.selection.stroke = Stroke::new(1.0, ACCENT);

        let r = radius(6);
        v.widgets.noninteractive.bg_fill = PANEL;
        v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, BORDER);
        v.widgets.noninteractive.fg_stroke = Stroke::new(1.0, TEXT);
        v.widgets.noninteractive.corner_radius = r;

        v.widgets.inactive.bg_fill = CONTROL;
        v.widgets.inactive.weak_bg_fill = CONTROL;
        v.widgets.inactive.bg_stroke = Stroke::new(1.0, Color32::from_rgb(0x3c, 0x3c, 0x47));
        v.widgets.inactive.fg_stroke = Stroke::new(1.0, TEXT);
        v.widgets.inactive.corner_radius = r;

        v.widgets.hovered.bg_fill = CONTROL_HOVER;
        v.widgets.hovered.weak_bg_fill = CONTROL_HOVER;
        v.widgets.hovered.bg_stroke = Stroke::NONE;
        v.widgets.hovered.fg_stroke = Stroke::new(1.0, Color32::WHITE);
        v.widgets.hovered.corner_radius = r;
        v.widgets.hovered.expansion = 0.0;

        v.widgets.active.bg_fill = CONTROL_HOVER;
        v.widgets.active.weak_bg_fill = CONTROL_HOVER;
        v.widgets.active.bg_stroke = Stroke::new(1.0, ACCENT);
        v.widgets.active.fg_stroke = Stroke::new(1.0, Color32::WHITE);
        v.widgets.active.corner_radius = r;
        v.widgets.active.expansion = 0.0;

        v.widgets.open.bg_fill = CONTROL_HOVER;
        v.widgets.open.weak_bg_fill = CONTROL_HOVER;
        v.widgets.open.bg_stroke = Stroke::new(1.0, BORDER);
        v.widgets.open.corner_radius = r;
    });
}
