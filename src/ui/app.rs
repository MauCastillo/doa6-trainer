//! Ventana gráfica del trainer implementada con egui y eframe 0.36.

use std::time::Duration;

use eframe::egui::{self, Color32, CornerRadius, Frame, Margin, RichText, Stroke, Vec2};

use crate::application::controller::{Controller, TrainerSnapshot};
use crate::application::steam_emu::{SteamEmu, SteamEmuStatus};
use crate::domain::cheat::{Cheat, CheatId, Hotkey};

pub struct TrainerApp {
    controller: Controller,
}

impl TrainerApp {
    pub fn new(controller: Controller) -> Self {
        Self { controller }
    }
}

impl eframe::App for TrainerApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // Redibuja periódicamente para reflejar el estado del juego y los atajos globales
        ui.ctx().request_repaint_after(Duration::from_millis(50));

        let snapshot = self.controller.snapshot();
        let catalog = self.controller.catalog();

        Frame::default()
            .fill(Color32::from_rgb(18, 18, 24))
            .inner_margin(Margin::same(16))
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing = Vec2::new(8.0, 10.0);

                // Cabecera principal
                render_header(ui, &snapshot, catalog.title, catalog.supported_versions);

                // Barra de error si ocurrió alguno
                if let Some(err) = &snapshot.last_error {
                    render_error_banner(ui, &self.controller, &err.to_string());
                }

                ui.separator();

                // Barra de estado de atajos globales
                render_hotkeys_bar(ui, &self.controller, &snapshot);

                ui.separator();

                // Categorías de trucos
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        render_cheat_group(
                            ui,
                            &self.controller,
                            &snapshot,
                            "Combate",
                            &["1", "2", "9", "0"],
                        );

                        render_cheat_group(
                            ui,
                            &self.controller,
                            &snapshot,
                            "Tiempo y Velocidad",
                            &["5", "7", "8"],
                        );

                        render_cheat_group(
                            ui,
                            &self.controller,
                            &snapshot,
                            "Puntuación y Desafíos",
                            &["4", "6"],
                        );

                        render_cheat_group(
                            ui,
                            &self.controller,
                            &snapshot,
                            "Desbloqueables y Economía",
                            &["C1", "3", "C2", "C3", "C4", "C5", "C6"],
                        );

                        render_persistent_section(ui);

                        ui.add_space(8.0);
                        ui.vertical_centered(|ui| {
                            ui.label(
                                RichText::new("Clean Architecture & Clean Code • Reescrito en Rust")
                                    .size(11.0)
                                    .color(Color32::from_rgb(100, 100, 120)),
                            );
                        });
                    });
            });
    }
}

fn render_header(ui: &mut egui::Ui, snapshot: &TrainerSnapshot, title: &str, versions: &str) {
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.heading(
                RichText::new(title)
                    .size(22.0)
                    .strong()
                    .color(Color32::from_rgb(240, 240, 255)),
            );
            ui.label(
                RichText::new(format!("{versions} • Plus 16 Trainer"))
                    .size(12.0)
                    .color(Color32::from_rgb(140, 140, 170)),
            );
        });

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            match snapshot.game_pid {
                Some(pid) => {
                    let text = format!("🟢 Conectado (PID: {pid})");
                    ui.label(
                        RichText::new(text)
                            .size(13.0)
                            .strong()
                            .color(Color32::from_rgb(80, 220, 120)),
                    );
                }
                None => {
                    ui.label(
                        RichText::new("🔴 Esperando DOA6.exe / DOA6LR.exe...")
                            .size(13.0)
                            .strong()
                            .color(Color32::from_rgb(250, 140, 60)),
                    );
                }
            }
        });
    });
}

fn render_error_banner(ui: &mut egui::Ui, controller: &Controller, error_msg: &str) {
    ui.add_space(4.0);
    Frame::default()
        .fill(Color32::from_rgb(60, 20, 24))
        .stroke(Stroke::new(1.0, Color32::from_rgb(200, 60, 80)))
        .corner_radius(CornerRadius::same(6))
        .inner_margin(Margin::symmetric(10, 6))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(format!("⚠️ {error_msg}"))
                        .size(12.0)
                        .color(Color32::from_rgb(255, 180, 190)),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(RichText::new("Cerrar").size(11.0)).clicked() {
                        controller.dismiss_error();
                    }
                });
            });
        });
}

fn render_hotkeys_bar(ui: &mut egui::Ui, controller: &Controller, snapshot: &TrainerSnapshot) {
    ui.horizontal(|ui| {
        let (status_text, status_color) = if snapshot.hotkeys_enabled {
            ("ACTIVADOS", Color32::from_rgb(80, 220, 120))
        } else {
            ("PAUSADOS", Color32::from_rgb(200, 80, 80))
        };

        ui.label(
            RichText::new("Atajos globales:")
                .size(12.0)
                .color(Color32::from_rgb(180, 180, 200)),
        );
        ui.label(RichText::new(status_text).size(12.0).strong().color(status_color));

        ui.label(
            RichText::new("(Ctrl+Shift+Inicio para pausar/reanudar)")
                .size(11.0)
                .color(Color32::from_rgb(120, 120, 140)),
        );

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let btn_label = if snapshot.hotkeys_enabled {
                "Pausar atajos"
            } else {
                "Reanudar atajos"
            };
            if ui.button(RichText::new(btn_label).size(11.0)).clicked() {
                controller.set_hotkeys_enabled(!snapshot.hotkeys_enabled);
            }
        });
    });
}

fn render_cheat_group(
    ui: &mut egui::Ui,
    controller: &Controller,
    snapshot: &TrainerSnapshot,
    title: &'static str,
    cheat_ids: &[&'static str],
) {
    ui.add_space(4.0);
    ui.label(
        RichText::new(title)
            .size(14.0)
            .strong()
            .color(Color32::from_rgb(170, 170, 220)),
    );

    Frame::default()
        .fill(Color32::from_rgb(24, 25, 34))
        .stroke(Stroke::new(1.0, Color32::from_rgb(45, 47, 62)))
        .corner_radius(CornerRadius::same(8))
        .inner_margin(Margin::same(8))
        .show(ui, |ui| {
            for &id_str in cheat_ids {
                let cheat_id = CheatId(id_str);
                if let Some(cheat) = controller.catalog().cheat(cheat_id) {
                    let is_active = snapshot.active.contains(&cheat_id);
                    render_cheat_row(ui, controller, cheat, is_active);
                }
            }
        });
}

fn render_cheat_row(ui: &mut egui::Ui, controller: &Controller, cheat: &Cheat, is_active: bool) {
    ui.horizontal(|ui| {
        // Badge del atajo de teclado
        let hotkey_str = format_hotkey(cheat.hotkey);
        Frame::default()
            .fill(Color32::from_rgb(34, 36, 48))
            .stroke(Stroke::new(1.0, Color32::from_rgb(60, 64, 85)))
            .corner_radius(CornerRadius::same(4))
            .inner_margin(Margin::symmetric(6, 3))
            .show(ui, |ui| {
                ui.label(
                    RichText::new(hotkey_str)
                        .size(11.0)
                        .monospace()
                        .color(Color32::from_rgb(200, 210, 240)),
                );
            });

        // Nombres en español e inglés
        ui.vertical(|ui| {
            ui.label(
                RichText::new(cheat.name.es)
                    .size(13.0)
                    .color(Color32::from_rgb(235, 235, 245)),
            );
            ui.label(
                RichText::new(cheat.name.en)
                    .size(10.5)
                    .color(Color32::from_rgb(130, 130, 160)),
            );
        });

        // Botón toggle a la derecha
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let (label, bg_color, text_color) = if is_active {
                ("ACTIVO", Color32::from_rgb(40, 130, 70), Color32::WHITE)
            } else {
                ("INACTIVO", Color32::from_rgb(38, 40, 52), Color32::from_rgb(160, 165, 185))
            };

            let button = egui::Button::new(
                RichText::new(label)
                    .size(11.0)
                    .strong()
                    .color(text_color),
            )
            .fill(bg_color)
            .min_size(Vec2::new(75.0, 24.0));

            if ui.add(button).clicked() {
                controller.toggle(cheat.id);
            }
        });
    });
}

fn format_hotkey(hotkey: Hotkey) -> String {
    if hotkey.ctrl {
        format!("Ctrl+Num {}", hotkey.numpad)
    } else {
        format!("Num {}", hotkey.numpad)
    }
}

fn render_persistent_section(ui: &mut egui::Ui) {
    ui.add_space(4.0);
    ui.label(
        RichText::new("Opciones Persistentes (Archivos / Disco)")
            .size(14.0)
            .strong()
            .color(Color32::from_rgb(170, 170, 220)),
    );

    Frame::default()
        .fill(Color32::from_rgb(24, 25, 34))
        .stroke(Stroke::new(1.0, Color32::from_rgb(45, 47, 62)))
        .corner_radius(CornerRadius::same(8))
        .inner_margin(Margin::same(8))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                Frame::default()
                    .fill(Color32::from_rgb(34, 36, 48))
                    .stroke(Stroke::new(1.0, Color32::from_rgb(60, 64, 85)))
                    .corner_radius(CornerRadius::same(4))
                    .inner_margin(Margin::symmetric(6, 3))
                    .show(ui, |ui| {
                        ui.label(
                            RichText::new("DISCO")
                                .size(11.0)
                                .monospace()
                                .color(Color32::from_rgb(200, 210, 240)),
                        );
                    });

                ui.vertical(|ui| {
                    ui.label(
                        RichText::new("Desbloquear todos los DLCs y Personajes")
                            .size(13.0)
                            .color(Color32::from_rgb(235, 235, 245)),
                    );
                    ui.label(
                        RichText::new("steam_emu.ini • Mai, Kula, Momiji, Rachel, trajes y modos")
                            .size(10.0)
                            .color(Color32::from_rgb(140, 140, 160)),
                    );
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let status = SteamEmu::status();
                    match status {
                        SteamEmuStatus::Unlocked(_) => {
                            let button = egui::Button::new(
                                RichText::new("ACTIVADO")
                                    .size(11.0)
                                    .strong()
                                    .color(Color32::WHITE),
                            )
                            .fill(Color32::from_rgb(40, 130, 70))
                            .min_size(Vec2::new(75.0, 24.0));
                            if ui.add(button).clicked() {
                                let _ = SteamEmu::toggle();
                            }
                        }
                        SteamEmuStatus::Locked(_) => {
                            let button = egui::Button::new(
                                RichText::new("DESACTIVADO")
                                    .size(11.0)
                                    .color(Color32::from_rgb(160, 165, 185)),
                            )
                            .fill(Color32::from_rgb(38, 40, 52))
                            .min_size(Vec2::new(75.0, 24.0));
                            if ui.add(button).clicked() {
                                let _ = SteamEmu::toggle();
                            }
                        }
                        SteamEmuStatus::NotFound => {
                            ui.label(
                                RichText::new("No detectado")
                                    .size(10.0)
                                    .color(Color32::from_rgb(160, 140, 100)),
                            );
                        }
                    }
                });
            });

            ui.add_space(4.0);
            ui.label(
                RichText::new("💡 Consejo: Al activar Dinero infinito (Ctrl+Num 1), las compras en el armario se guardan para siempre en SYSTEMDATA.")
                    .size(10.5)
                    .color(Color32::from_rgb(130, 150, 180)),
            );
        });
}
