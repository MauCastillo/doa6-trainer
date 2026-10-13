//! Capa de presentación (UI) construida con eframe y egui.

pub mod app;
pub mod i18n;
pub mod settings;
pub mod theme;
pub mod widgets;

pub use app::TrainerApp;

/// Nombre visible de la aplicación (ventana y encabezado).
pub const APP_NAME: &str = "DOA6 Trainer +17";
