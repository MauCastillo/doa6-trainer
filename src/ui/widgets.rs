//! Small custom widgets: animated switch, "?" help button, hotkey chip and status pill.

use eframe::egui::{
    self, Align2, Color32, CornerRadius, FontId, Response, RichText, Sense, Stroke, StrokeKind, Vec2, WidgetInfo,
    WidgetType,
};

use super::theme;

/// On/off switch. Paints itself faded when the surrounding `Ui` is disabled.
pub fn switch(ui: &mut egui::Ui, on: bool, label: &str) -> Response {
    let size = Vec2::new(42.0, 24.0);
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    response.widget_info(|| WidgetInfo::selected(WidgetType::Checkbox, ui.is_enabled(), on, label));

    if ui.is_rect_visible(rect) {
        let t = ui.ctx().animate_bool_responsive(response.id, on);
        let mut track = theme::MUTED.lerp_to_gamma(theme::PRIMARY, t);
        if response.hovered() {
            track = track.lerp_to_gamma(Color32::WHITE, 0.08);
        }
        let border = if response.has_focus() { Stroke::new(2.0, theme::SECONDARY) } else { Stroke::new(1.0, theme::BORDER) };
        let painter = ui.painter();
        let radius = rect.height() / 2.0;
        painter.rect_filled(rect, radius, track);
        painter.rect_stroke(rect, radius, border, StrokeKind::Inside);

        let knob_x = egui::lerp((rect.left() + radius)..=(rect.right() - radius), t);
        painter.circle_filled(egui::pos2(knob_x, rect.center().y), radius - 4.0, Color32::WHITE);
    }

    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// Round "?" button. Shows `help` as a tooltip on hover and returns the response so
/// the caller can also expand the text inline on click (help must not rely on hover only).
pub fn help_button(ui: &mut egui::Ui, open: bool, help: &str, accessible_label: &str) -> Response {
    let size = Vec2::splat(20.0);
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    response.widget_info(|| WidgetInfo::selected(WidgetType::Button, true, open, accessible_label));

    if ui.is_rect_visible(rect) {
        let highlighted = open || response.hovered() || response.has_focus();
        let (fill, stroke, text) = if highlighted {
            (theme::PRIMARY, theme::SECONDARY, Color32::WHITE)
        } else {
            (theme::MUTED, theme::BORDER, theme::MUTED_FOREGROUND)
        };
        let painter = ui.painter();
        let center = rect.center();
        painter.circle_filled(center, 9.0, fill);
        painter.circle_stroke(center, 9.0, Stroke::new(1.0, stroke));
        painter.text(center, Align2::CENTER_CENTER, "?", FontId::proportional(12.5), text);
    }

    response.on_hover_cursor(egui::CursorIcon::Help).on_hover_ui(|ui| {
        ui.set_max_width(320.0);
        ui.label(RichText::new(help).size(13.0));
    })
}

/// Expanded help text shown under an option after clicking its "?".
pub fn help_panel(ui: &mut egui::Ui, text: &str) {
    egui::Frame::default()
        .fill(theme::INFO_BG)
        .stroke(Stroke::new(1.0, theme::BORDER_STRONG))
        .corner_radius(CornerRadius::same(theme::RADIUS_SM))
        .inner_margin(egui::Margin::symmetric(10, 8))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(RichText::new(text).size(13.0).color(theme::FOREGROUND));
        });
}

/// Fixed-width chip with a keyboard shortcut, so option names line up.
pub fn hotkey_chip(ui: &mut egui::Ui, text: &str, tooltip: &str) -> Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::new(86.0, 24.0), Sense::hover());
    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        painter.rect_filled(rect, theme::RADIUS_SM, theme::BACKGROUND);
        painter.rect_stroke(rect, theme::RADIUS_SM, Stroke::new(1.0, theme::BORDER), StrokeKind::Inside);
        painter.text(rect.center(), Align2::CENTER_CENTER, text, FontId::monospace(12.0), theme::SECONDARY);
    }
    response.on_hover_text(tooltip)
}

/// Coloured dot followed by a label; meaning is carried by the text, not only the colour.
pub fn status_pill(ui: &mut egui::Ui, color: Color32, text: &str) -> Response {
    egui::Frame::default()
        .fill(color.gamma_multiply(0.14))
        .stroke(Stroke::new(1.0, color.gamma_multiply(0.6)))
        .corner_radius(CornerRadius::same(255))
        .inner_margin(egui::Margin::symmetric(10, 4))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                let (dot, _) = ui.allocate_exact_size(Vec2::splat(10.0), Sense::hover());
                ui.painter().circle_filled(dot.center(), 4.5, color);
                ui.label(RichText::new(text).size(13.0).strong().color(color));
            });
        })
        .response
}

/// Card container used by every section.
pub fn card<R>(ui: &mut egui::Ui, add_contents: impl FnOnce(&mut egui::Ui) -> R) -> R {
    egui::Frame::default()
        .fill(theme::CARD)
        .stroke(Stroke::new(1.0, theme::BORDER))
        .corner_radius(CornerRadius::same(theme::RADIUS_MD))
        .inner_margin(egui::Margin::symmetric(12, 10))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            add_contents(ui)
        })
        .inner
}
