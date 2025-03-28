#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use egui::{Vec2, ViewportBuilder};
use log::info;

mod app;
mod llm;
mod ui;
mod config;
mod security;
mod db_element;
mod utils;

fn main() -> eframe::Result<()> {

    let native_options = eframe::NativeOptions {
        renderer: eframe::Renderer::Wgpu,
        viewport: ViewportBuilder::default().with_inner_size(Vec2::new(800.0, 600.0)),
        ..Default::default()
    };

    info!("Starting the application");
    eframe::run_native(
        "Database Query Assistant",
        native_options,
        Box::new(|cc|
            Ok( Box::new(app::DBQueryApp::new(cc)))
        )
    )
}