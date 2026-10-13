//! Design tokens and global egui style for the trainer.
//!
//! Palette from the UI/UX Pro Max gaming design system ("neon purple + rose
//! action") tuned for a dark desktop utility: every text/background pair keeps
//! at least 4.5:1 contrast.

use eframe::egui::{self, Color32, CornerRadius, FontId, Stroke, TextStyle, Theme, Vec2};

pub const BACKGROUND: Color32 = Color32::from_rgb(0x0F, 0x0F, 0x23);
pub const CARD: Color32 = Color32::from_rgb(0x1E, 0x1C, 0x35);
pub const CARD_HOVER: Color32 = Color32::from_rgb(0x26, 0x23, 0x44);
pub const MUTED: Color32 = Color32::from_rgb(0x27, 0x27, 0x3B);
pub const BORDER: Color32 = Color32::from_rgb(0x34, 0x30, 0x5A);
pub const BORDER_STRONG: Color32 = Color32::from_rgb(0x4C, 0x1D, 0x95);

pub const FOREGROUND: Color32 = Color32::from_rgb(0xE2, 0xE8, 0xF0);
pub const MUTED_FOREGROUND: Color32 = Color32::from_rgb(0x94, 0xA3, 0xB8);

pub const PRIMARY: Color32 = Color32::from_rgb(0x7C, 0x3A, 0xED);
pub const PRIMARY_HOVER: Color32 = Color32::from_rgb(0x8B, 0x5C, 0xF6);
pub const SECONDARY: Color32 = Color32::from_rgb(0xA7, 0x8B, 0xFA);
pub const ACCENT: Color32 = Color32::from_rgb(0xF4, 0x3F, 0x5E);

pub const SUCCESS: Color32 = Color32::from_rgb(0x22, 0xC5, 0x5E);
pub const WARNING: Color32 = Color32::from_rgb(0xF5, 0x9E, 0x0B);
pub const DANGER: Color32 = Color32::from_rgb(0xEF, 0x44, 0x44);
pub const DANGER_BG: Color32 = Color32::from_rgb(0x3B, 0x14, 0x22);
pub const INFO_BG: Color32 = Color32::from_rgb(0x1B, 0x1F, 0x3F);

pub const RADIUS_SM: u8 = 6;
pub const RADIUS_MD: u8 = 10;

/// Installs the dark theme, text sizes and spacing for the whole app.
pub fn apply(ctx: &egui::Context) {
    ctx.set_theme(Theme::Dark);
    ctx.style_mut_of(Theme::Dark, |style| {
        style.text_styles = [
            (TextStyle::Heading, FontId::proportional(22.0)),
            (TextStyle::Body, FontId::proportional(14.0)),
            (TextStyle::Button, FontId::proportional(13.5)),
            (TextStyle::Small, FontId::proportional(12.0)),
            (TextStyle::Monospace, FontId::monospace(12.5)),
        ]
        .into();

        style.spacing.item_spacing = Vec2::new(8.0, 8.0);
        style.spacing.button_padding = Vec2::new(12.0, 6.0);
        style.spacing.interact_size.y = 28.0;

        let visuals = &mut style.visuals;
        visuals.dark_mode = true;
        visuals.panel_fill = BACKGROUND;
        visuals.window_fill = CARD;
        visuals.window_stroke = Stroke::new(1.0, BORDER);
        visuals.extreme_bg_color = BACKGROUND;
        visuals.faint_bg_color = CARD;
        visuals.override_text_color = None;
        visuals.hyperlink_color = SECONDARY;
        visuals.warn_fg_color = WARNING;
        visuals.error_fg_color = DANGER;
        visuals.selection.bg_fill = PRIMARY;
        visuals.selection.stroke = Stroke::new(1.0, FOREGROUND);

        let widgets = &mut visuals.widgets;
        widgets.noninteractive.bg_fill = CARD;
        widgets.noninteractive.bg_stroke = Stroke::new(1.0, BORDER);
        widgets.noninteractive.fg_stroke = Stroke::new(1.0, FOREGROUND);

        widgets.inactive.bg_fill = MUTED;
        widgets.inactive.weak_bg_fill = MUTED;
        widgets.inactive.bg_stroke = Stroke::new(1.0, BORDER);
        widgets.inactive.fg_stroke = Stroke::new(1.0, FOREGROUND);

        widgets.hovered.bg_fill = CARD_HOVER;
        widgets.hovered.weak_bg_fill = CARD_HOVER;
        widgets.hovered.bg_stroke = Stroke::new(1.0, SECONDARY);
        widgets.hovered.fg_stroke = Stroke::new(1.5, Color32::WHITE);

        // Visible focus ring for keyboard navigation (Tab / Space)
        widgets.active.bg_fill = PRIMARY;
        widgets.active.weak_bg_fill = PRIMARY;
        widgets.active.bg_stroke = Stroke::new(2.0, SECONDARY);
        widgets.active.fg_stroke = Stroke::new(1.5, Color32::WHITE);

        for state in [&mut widgets.noninteractive, &mut widgets.inactive, &mut widgets.hovered, &mut widgets.active] {
            state.corner_radius = CornerRadius::same(RADIUS_SM);
        }
    });
}
