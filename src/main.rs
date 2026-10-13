use eframe::egui::Vec2;

use doa6_trainer::application::controller::Controller;
use doa6_trainer::domain::doa6;
use doa6_trainer::infrastructure::{default_hotkey_listener, default_process_finder};
use doa6_trainer::ui::TrainerApp;

fn main() -> eframe::Result<()> {
    let catalog = doa6::catalog();
    let mut controller = Controller::new(catalog);

    // Arrancar hilo en segundo plano de detección del juego y atajos globales
    let finder = default_process_finder();
    let hotkeys = default_hotkey_listener();
    controller.start(finder, hotkeys, || {});

    let native_options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size(Vec2::new(540.0, 720.0))
            .with_min_inner_size(Vec2::new(480.0, 520.0))
            .with_title("Dead or Alive 6 Plus 16 Trainer"),
        ..Default::default()
    };

    eframe::run_native(
        "Dead or Alive 6 Plus 16 Trainer",
        native_options,
        Box::new(|_cc| Ok(Box::new(TrainerApp::new(controller)))),
    )
}
