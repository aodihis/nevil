use egui::{Vec2, ViewportBuilder};

mod app;
mod llm;
mod ui;
mod config;
mod security;
mod db;

fn main() -> eframe::Result<()> {

    let native_options = eframe::NativeOptions {
        renderer: eframe::Renderer::Wgpu,
        viewport: ViewportBuilder::default().with_inner_size(Vec2::new(800.0, 600.0)),
        ..Default::default()
    };

    eframe::run_native(
        "Database Query Assistant",
        native_options,
        Box::new(|cc|
            Ok( Box::new(app::DBQueryApp::new(cc)))
        )
    )
}