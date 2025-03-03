mod app;
mod db;
mod llm;
mod ui;
mod config;
mod security;

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
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