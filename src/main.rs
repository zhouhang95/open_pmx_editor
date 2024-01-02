#![warn(clippy::all, rust_2018_idioms)]
//#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

fn main() -> eframe::Result<()> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let native_options = eframe::NativeOptions {
        depth_buffer: 32,
        viewport: egui::ViewportBuilder::default()
            .with_drag_and_drop(true),
        ..Default::default()
    };
    eframe::run_native(
        "Open MMD Editor",
        native_options,
        Box::new(|cc| Box::new(eframe_template::TemplateApp::new(cc))),
    )
}

