//! Main trainer window built with egui / eframe 0.36.

use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

use eframe::egui::{self, Align, Color32, CornerRadius, Frame, Layout, Margin, RichText, Stroke};

use super::i18n::{self, Language, SECTIONS, Strings};
use super::settings::Settings;
use super::theme;
use super::{APP_NAME, widgets};
use crate::application::controller::{Controller, TrainerSnapshot};
use crate::application::steam_emu::{SteamEmu, SteamEmuStatus};
use crate::domain::cheat::{Cheat, CheatId, Hotkey};

const DLC_REFRESH_INTERVAL: Duration = Duration::from_secs(1);
const APPLIED_FEEDBACK: Duration = Duration::from_secs(2);

pub struct TrainerApp {
    controller: Controller,
    settings: Settings,
    /// Help panels the user expanded with "?", by key.
    open_help: HashSet<String>,
    /// When each one-shot cheat was last applied successfully, for the "Applied" feedback.
    applied_at: HashMap<CheatId, Instant>,
    /// steam_emu.ini is read once per second instead of on every frame.
    dlc_status: SteamEmuStatus,
    dlc_checked: Instant,
    dlc_error: Option<String>,
    /// High-resolution mascot illustration texture for the header avatar.
    mascot_texture: Option<egui::TextureHandle>,
}

impl TrainerApp {
    pub fn new(cc: &eframe::CreationContext<'_>, controller: Controller) -> Self {
        theme::apply(&cc.egui_ctx);
        let mascot_texture = eframe::icon_data::from_png_bytes(include_bytes!("../../assets/icon_master.png"))
            .ok()
            .map(|icon_data| {
                let color_image: egui::ColorImage = icon_data.into();
                cc.egui_ctx.load_texture("mascot_icon", color_image, egui::TextureOptions::LINEAR)
            });

        Self {
            controller,
            settings: Settings::load(),
            open_help: HashSet::new(),
            applied_at: HashMap::new(),
            dlc_status: SteamEmu::status(),
            dlc_checked: Instant::now(),
            dlc_error: None,
            mascot_texture,
        }
    }

    fn language(&self) -> Language {
        self.settings.language
    }

    fn set_language(&mut self, language: Language) {
        if self.settings.language != language {
            self.settings.language = language;
            self.settings.save();
        }
    }

    /// "?" button plus, when expanded, the help text below it.
    fn help_toggle(&mut self, ui: &mut egui::Ui, key: &str, help: &str) -> bool {
        let open = self.open_help.contains(key);
        let label = self.language().strings().help_prefix;
        if widgets::help_button(ui, open, help, label).clicked() && !self.open_help.remove(key) {
            self.open_help.insert(key.to_owned());
        }
        self.open_help.contains(key)
    }

    fn refresh_dlc_status(&mut self, force: bool) {
        if force || self.dlc_checked.elapsed() >= DLC_REFRESH_INTERVAL {
            self.dlc_status = SteamEmu::status();
            self.dlc_checked = Instant::now();
        }
    }
}

impl eframe::App for TrainerApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // Periodic repaint to reflect game state and global hotkeys
        ui.ctx().request_repaint_after(Duration::from_millis(50));
        self.refresh_dlc_status(false);

        let snapshot = self.controller.snapshot();

        Frame::default().fill(theme::BACKGROUND).inner_margin(Margin::same(16)).show(ui, |ui| {
            ui.set_width(ui.available_width());
            self.render_header(ui, &snapshot);
            ui.add_space(4.0);

            if let Some(error) = &snapshot.last_error {
                let message = i18n::error_message(self.language(), error);
                if error_banner(ui, &message, self.language().strings()) {
                    self.controller.dismiss_error();
                }
            }
            if let Some(message) = self.dlc_error.clone()
                && error_banner(ui, &message, self.language().strings())
            {
                self.dlc_error = None;
            }
            if snapshot.game_pid.is_none() {
                info_banner(ui, self.language().strings().waiting_banner);
            }

            self.render_toolbar(ui, &snapshot);
            ui.add_space(4.0);

            egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                for section in &SECTIONS {
                    self.render_section(ui, &snapshot, section);
                    ui.add_space(10.0);
                }
                self.render_persistent_section(ui);
                ui.add_space(12.0);
                self.render_footer(ui);
            });
        });
    }
}

impl TrainerApp {
    fn render_header(&mut self, ui: &mut egui::Ui, snapshot: &TrainerSnapshot) {
        let language = self.language();
        let text = language.strings();
        let title = self.controller.catalog().title;

        ui.horizontal(|ui| {
            if let Some(texture) = &self.mascot_texture {
                let size = egui::vec2(52.0, 52.0);
                let image = egui::Image::new(texture).fit_to_exact_size(size);
                ui.add(image).on_hover_text("DOA6 Mascot");
                ui.add_space(4.0);
            }

            ui.vertical(|ui| {
                ui.label(RichText::new(APP_NAME).size(24.0).strong().color(theme::FOREGROUND));
                ui.label(RichText::new(format!("{title} · {}", text.subtitle)).size(12.5).color(theme::MUTED_FOREGROUND));
            });

            ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                self.help_toggle(ui, "language", text.language_help);
                for option in Language::ALL.into_iter().rev() {
                    let selected = option == language;
                    let label = RichText::new(option.native_name()).size(13.0).color(if selected {
                        Color32::WHITE
                    } else {
                        theme::MUTED_FOREGROUND
                    });
                    let button = egui::Button::selectable(selected, label).corner_radius(CornerRadius::same(theme::RADIUS_SM));
                    if ui.add(button).on_hover_text(text.language_label).clicked() {
                        self.set_language(option);
                    }
                }
            });
        });

        if self.open_help.contains("language") {
            widgets::help_panel(ui, text.language_help);
        }

        ui.add_space(6.0);
        ui.horizontal(|ui| {
            match snapshot.game_pid {
                Some(pid) => {
                    widgets::status_pill(ui, theme::SUCCESS, &format!("{} · PID {pid}", text.status_connected));
                }
                None => {
                    widgets::status_pill(ui, theme::WARNING, text.status_waiting);
                }
            }
            self.help_toggle(ui, "status", text.status_help);
        });
        if self.open_help.contains("status") {
            widgets::help_panel(ui, text.status_help);
        }
    }

    fn render_toolbar(&mut self, ui: &mut egui::Ui, snapshot: &TrainerSnapshot) {
        let text = self.language().strings();
        widgets::card(ui, |ui| {
            ui.horizontal(|ui| {
                let enabled = snapshot.hotkeys_enabled;
                if widgets::switch(ui, enabled, text.hotkeys_label).clicked() {
                    self.controller.set_hotkeys_enabled(!enabled);
                }
                ui.label(RichText::new(text.hotkeys_label).size(14.0).color(theme::FOREGROUND));
                let (state, color) =
                    if enabled { (text.hotkeys_on, theme::SUCCESS) } else { (text.hotkeys_off, theme::WARNING) };
                ui.label(RichText::new(state).size(12.5).strong().color(color));
                self.help_toggle(ui, "hotkeys", text.hotkeys_help);

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let active = snapshot.active.len();
                    let button = egui::Button::new(RichText::new(text.disable_all).size(13.0));
                    if ui.add_enabled(active > 0, button).on_hover_text(text.disable_all_help).clicked() {
                        for id in &snapshot.active {
                            self.controller.toggle(*id);
                        }
                    }
                    let color = if active > 0 { theme::SECONDARY } else { theme::MUTED_FOREGROUND };
                    ui.label(RichText::new(format!("{active} {}", text.active_count)).size(12.5).color(color));
                });
            });
            ui.label(RichText::new(text.hotkeys_shortcut).size(12.0).color(theme::MUTED_FOREGROUND));
            if self.open_help.contains("hotkeys") {
                widgets::help_panel(ui, text.hotkeys_help);
            }
        });
    }

    fn render_section(&mut self, ui: &mut egui::Ui, snapshot: &TrainerSnapshot, section: &i18n::Section) {
        let language = self.language();
        let key = format!("section:{}", section.title.en);

        ui.horizontal(|ui| {
            section_title(ui, language.pick(section.title));
            self.help_toggle(ui, &key, language.pick(section.help));
        });
        if self.open_help.contains(&key) {
            widgets::help_panel(ui, language.pick(section.help));
            ui.add_space(4.0);
        }

        let cheats: Vec<Cheat> = section
            .cheat_ids
            .iter()
            .filter_map(|id| self.controller.catalog().cheat(CheatId(id)).cloned())
            .collect();

        widgets::card(ui, |ui| {
            for (index, cheat) in cheats.iter().enumerate() {
                if index > 0 {
                    row_separator(ui);
                }
                self.render_cheat_row(ui, snapshot, cheat);
            }
        });
    }

    fn render_cheat_row(&mut self, ui: &mut egui::Ui, snapshot: &TrainerSnapshot, cheat: &Cheat) {
        let language = self.language();
        let text = language.strings();
        let attached = snapshot.game_pid.is_some();
        let is_active = snapshot.active.contains(&cheat.id);
        let name = language.pick(cheat.name);
        let help = language.pick(cheat.description);
        let key = format!("cheat:{}", cheat.id.0);

        ui.horizontal(|ui| {
            ui.set_min_height(32.0);
            widgets::hotkey_chip(ui, &format_hotkey(cheat.hotkey), text.hotkey_help);

            let name_color = if is_active { Color32::WHITE } else { theme::FOREGROUND };
            ui.label(RichText::new(name).size(14.0).color(name_color));
            self.help_toggle(ui, &key, help);

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.add_enabled_ui(attached, |ui| {
                    if cheat.effect.is_one_shot() {
                        self.render_apply_button(ui, cheat, name);
                    } else {
                        let state = if is_active { text.switch_on } else { text.switch_off };
                        let color = if is_active { theme::SECONDARY } else { theme::MUTED_FOREGROUND };
                        if widgets::switch(ui, is_active, name).clicked() {
                            self.controller.toggle(cheat.id);
                        }
                        ui.label(RichText::new(state).size(12.0).strong().color(color));
                    }
                })
                .response
                .on_disabled_hover_text(text.requires_game);
            });
        });

        if self.open_help.contains(&key) {
            widgets::help_panel(ui, help);
        }
    }

    fn render_apply_button(&mut self, ui: &mut egui::Ui, cheat: &Cheat, name: &str) {
        let text = self.language().strings();
        let recently_applied = self.applied_at.get(&cheat.id).is_some_and(|at| at.elapsed() < APPLIED_FEEDBACK);

        let button = egui::Button::new(RichText::new(text.apply).size(13.0).strong().color(Color32::WHITE))
            .fill(theme::PRIMARY)
            .min_size(egui::vec2(84.0, 28.0));
        if ui.add(button).on_hover_text(name).clicked() {
            self.controller.toggle(cheat.id);
            if self.controller.snapshot().last_error.is_none() {
                self.applied_at.insert(cheat.id, Instant::now());
            }
        }
        if recently_applied {
            ui.label(RichText::new(format!("✔ {}", text.applied)).size(12.5).strong().color(theme::SUCCESS));
        }
    }

    fn render_persistent_section(&mut self, ui: &mut egui::Ui) {
        let text = self.language().strings();

        ui.horizontal(|ui| {
            section_title(ui, text.persistent_title);
            self.help_toggle(ui, "persistent", text.persistent_help);
        });
        if self.open_help.contains("persistent") {
            widgets::help_panel(ui, text.persistent_help);
            ui.add_space(4.0);
        }

        widgets::card(ui, |ui| {
            ui.horizontal(|ui| {
                ui.set_min_height(32.0);
                widgets::hotkey_chip(ui, text.disk_chip, text.persistent_help);
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(text.dlc_name).size(14.0).color(theme::FOREGROUND));
                        self.help_toggle(ui, "dlc", text.dlc_help);
                    });
                    ui.label(RichText::new(text.dlc_detail).size(12.0).color(theme::MUTED_FOREGROUND));
                });

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| match &self.dlc_status {
                    SteamEmuStatus::Unlocked(_) | SteamEmuStatus::Locked(_) => {
                        let on = matches!(self.dlc_status, SteamEmuStatus::Unlocked(_));
                        if widgets::switch(ui, on, text.dlc_name).clicked() {
                            self.dlc_error = SteamEmu::toggle().err().map(|error| i18n::dlc_error(self.language(), &error));
                            self.refresh_dlc_status(true);
                        }
                        let (state, color) =
                            if on { (text.switch_on, theme::SECONDARY) } else { (text.switch_off, theme::MUTED_FOREGROUND) };
                        ui.label(RichText::new(state).size(12.0).strong().color(color));
                    }
                    SteamEmuStatus::NotFound => {
                        ui.label(RichText::new(text.dlc_not_found).size(12.5).color(theme::WARNING))
                            .on_hover_text(text.dlc_not_found_help);
                    }
                });
            });
            if self.open_help.contains("dlc") {
                widgets::help_panel(ui, text.dlc_help);
            }
            if self.dlc_status == SteamEmuStatus::NotFound {
                ui.label(RichText::new(text.dlc_not_found_help).size(12.5).color(theme::MUTED_FOREGROUND));
            }

            row_separator(ui);
            ui.label(RichText::new(text.money_tip).size(12.5).color(theme::SECONDARY));
        });
    }

    fn render_footer(&self, ui: &mut egui::Ui) {
        let text = self.language().strings();
        ui.vertical_centered(|ui| {
            ui.label(RichText::new(text.footer_restore).size(12.5).color(theme::MUTED_FOREGROUND));
            ui.label(RichText::new(text.footer_credits).size(12.0).color(theme::MUTED_FOREGROUND.gamma_multiply(0.8)));
        });
    }
}

fn section_title(ui: &mut egui::Ui, title: &str) {
    ui.label(RichText::new(title.to_uppercase()).size(13.0).strong().color(theme::SECONDARY));
}

fn row_separator(ui: &mut egui::Ui) {
    ui.add_space(2.0);
    let width = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, 1.0), egui::Sense::hover());
    ui.painter().rect_filled(rect, 0.0, theme::BORDER);
    ui.add_space(2.0);
}

fn info_banner(ui: &mut egui::Ui, message: &str) {
    Frame::default()
        .fill(theme::INFO_BG)
        .stroke(Stroke::new(1.0, theme::BORDER_STRONG))
        .corner_radius(CornerRadius::same(theme::RADIUS_SM))
        .inner_margin(Margin::symmetric(12, 8))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(RichText::new(message).size(13.0).color(theme::FOREGROUND));
        });
    ui.add_space(4.0);
}

/// Returns `true` when the user dismisses the banner.
fn error_banner(ui: &mut egui::Ui, message: &str, text: &Strings) -> bool {
    let mut dismissed = false;
    Frame::default()
        .fill(theme::DANGER_BG)
        .stroke(Stroke::new(1.0, theme::DANGER))
        .corner_radius(CornerRadius::same(theme::RADIUS_SM))
        .inner_margin(Margin::symmetric(12, 8))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    dismissed = ui.button(RichText::new(text.dismiss).size(12.5)).clicked();
                    ui.with_layout(Layout::left_to_right(Align::Center).with_main_wrap(true), |ui| {
                        ui.label(RichText::new(message).size(13.0).color(Color32::from_rgb(0xFE, 0xCD, 0xD3)));
                    });
                });
            });
        });
    ui.add_space(4.0);
    dismissed
}

fn format_hotkey(hotkey: Hotkey) -> String {
    if hotkey.ctrl { format!("Ctrl+Num {}", hotkey.numpad) } else { format!("Num {}", hotkey.numpad) }
}
