// Sin ventana de consola en Windows (las builds de depuración la mantienen para ver logs)
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui::Vec2;

use doa6_trainer::application::controller::Controller;
use doa6_trainer::domain::doa6;
use doa6_trainer::infrastructure::{default_hotkey_listener, default_process_finder};
use doa6_trainer::ui::{APP_NAME, TrainerApp};

fn main() -> eframe::Result<()> {
    let catalog = doa6::catalog();
    let mut controller = Controller::new(catalog);

    // Arrancar hilo en segundo plano de detección del juego y atajos globales
    let finder = default_process_finder();
    let hotkeys = default_hotkey_listener();
    controller.start(finder, hotkeys, || {});

    let icon_data = eframe::icon_data::from_png_bytes(include_bytes!("../assets/icon.png")).ok();

    let mut viewport = eframe::egui::ViewportBuilder::default()
        .with_inner_size(Vec2::new(600.0, 820.0))
        .with_min_inner_size(Vec2::new(520.0, 560.0))
        .with_title(APP_NAME);

    if let Some(icon) = icon_data {
        viewport = viewport.with_icon(icon);
    }

    let native_options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        APP_NAME,
        native_options,
        Box::new(|cc| Ok(Box::new(TrainerApp::new(cc, controller)))),
    )
}
