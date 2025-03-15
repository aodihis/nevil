use crate::app::{AppMode, AppState};
use crate::config::{DbConnection, DbType};
use egui::{Context, TextEdit, Window};
use uuid::Uuid;

#[derive(Clone)]
pub struct Connection {
    pub uuid: Uuid,
    pub is_new: bool,
    pub name: String,
    pub db_type: DbType,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub database: String,
    pub password: String,
    pub success_message: Option<String>,
    pub error_message: Option<String>,

}

impl Connection {
    pub fn new() -> Self {
        Connection {
            uuid: Uuid::new_v4(),
            is_new: true,
            name: "New Connection".to_string(),
            db_type: DbType::MySQL,
            host: "localhost".to_string(),
            port: 3306,
            username: "root".to_string(),
            database: "".to_string(),
            password: "".to_string(),
            success_message: None,
            error_message: None,
        }
    }
}
pub fn connection_ui(ctx: &Context, app_state: &mut AppState) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading(if !app_state.connection.is_new { "Edit Connection" } else { "New Connection" });
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            ui.label("Connection Name:");
            ui.text_edit_singleline(&mut app_state.connection.name);
        });

        ui.horizontal(|ui| {
            ui.label("Database Type:");
            ui.radio_value(&mut app_state.connection.db_type, DbType::MySQL, "MySQL");
            ui.radio_value(&mut app_state.connection.db_type, DbType::PostgreSQL, "PostgreSQL");
        });

        ui.horizontal(|ui| {
            ui.label("Host:");
            ui.text_edit_singleline(&mut app_state.connection.host);
        });

        ui.horizontal(|ui| {
            ui.label("Port:");
            let mut port_str = app_state.connection.port.to_string();
            ui.text_edit_singleline(&mut port_str);
            if let Ok(port) = port_str.parse::<u16>() {
                app_state.connection.port = port;
            }
        });

        ui.horizontal(|ui| {
            ui.label("Database Name:");
            ui.text_edit_singleline(&mut app_state.connection.database);
        });

        ui.horizontal(|ui| {
            ui.label("Username:");
            ui.text_edit_singleline(&mut app_state.connection.username);
        });

        ui.horizontal(|ui| {
            ui.label("Password:");
            ui.add(TextEdit::singleline(&mut app_state.connection.password).password(true));
        });

        ui.add_space(20.0);

        ui.horizontal(|ui| {
            if ui.button("Cancel").clicked() {
                app_state.mode = AppMode::Connections;
            }

            if app_state.connection.is_new {
                if ui.button("Remove Connection").clicked() {
                    app_state.mode = AppMode::Home;
                }
            }
            if ui.button("Test Connection").clicked() {
                // Create a clone of connection for testing
                let test_connection = DbConnection {
                    uuid: Uuid::new_v4(),
                    name: "".to_string(),
                    db_type: app_state.connection.db_type.clone(),
                    host: app_state.connection.host.clone(),
                    port: app_state.connection.port,
                    username: app_state.connection.username.clone(),
                    database: app_state.connection.database.clone(),
                };
                let db_manager = app_state.db_manager.clone();

                // Test connection asynchronously
                app_state.connection.error_message = None;
                app_state.connection.success_message = Some("Testing connection...".to_string());

                let ctx = ctx.clone();
                let password = app_state.connection.password.clone();
                // This would be better with a callback mechanism
                app_state.runtime.spawn(async move {
                    if let Err(err) = db_manager.connect(&test_connection, Some(password)).await {
                        let res = format!("Connection test failed: {}", err);
                        eprintln!("{}", res);
                        Window::new("DB Connection Result")
                            .collapsible(false)
                            .resizable(false)
                            .auto_sized()
                            .anchor(egui::Align2::CENTER_TOP, egui::Vec2::new(0.0, 20.0))
                            .show(&ctx, |ui| {
                                ui.label(res);
                            });

                        // In a complete implementation, we would use a message passing system to update the UI
                    } else {
                        eprintln!("Connection test successful");
                    }
                });
            }

            if ui.button("Save Connection").clicked() {
                app_state.connection.error_message = None;
                app_state.connection.success_message = None;
                if app_state.connection.name.trim().is_empty() {
                    app_state.connection.error_message = Some("Connection name cannot be empty".to_string());
                } else if app_state.connection.database.trim().is_empty() {
                    app_state.connection.error_message = Some("Database name cannot be empty".to_string());
                } else {
                    app_state.save_connection();
                }
            }
        });

        // Display error/success messages
        ui.add_space(10.0);
        if let Some(ref err) = app_state.connection.error_message {
            ui.colored_label(egui::Color32::RED, err);
        }

        if let Some(ref success) = app_state.connection.success_message {
            ui.colored_label(egui::Color32::GREEN, success);
        }

        // Add AI-assisted setup option
        if app_state.llm_client.is_some() {
            ui.add_space(20.0);
            ui.separator();
            ui.add_space(10.0);

            ui.heading("AI-Assisted Setup");
            ui.label("Need help setting up your connection? The AI can help you configure it.");

            ui.add_space(5.0);

            if ui.button("Help me set up this connection").clicked() {
                // This would call an AI-assisted setup function
                // For now, we'll just show a placeholder message
                app_state.success_info = Some("AI-assisted setup would launch here".to_string());

                // In a complete implementation, this would:
                // 1. Launch a dialog to ask the user for high-level info about their database
                // 2. Use the LLM to generate connection parameters
                // 3. Pre-fill the form with these parameters
            }
        }

    });
}