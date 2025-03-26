use crate::app::{AppMode, AppState};
use crate::config::{DbConnection, DbType};
use egui::{Context, TextEdit, Window};
use log::info;
use uuid::Uuid;

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
    success_message: Option<String>,
    error_message: Option<String>,
    loading_message: Option<String>,
    confirm_delete: bool,
    rx: Option<tokio::sync::mpsc::Receiver<Result<(), String>>>,
}

impl Clone for Connection {
    fn clone(&self) -> Self {
        Self {
            uuid: self.uuid,
            is_new: self.is_new,
            name: self.name.clone(),
            db_type: self.db_type.clone(),
            host: self.host.clone(),
            port: self.port,
            username: self.username.clone(),
            database: self.database.clone(),
            password: self.password.clone(),
            success_message: None,
            error_message: None,
            loading_message: None,
            confirm_delete: false,
            rx: None,
        }
    }
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
            loading_message: None,
            confirm_delete: false,
            rx: None,
        }
    }
}
pub fn connection_ui(ctx: &Context, app_state: &mut AppState) {
    info!("Rending connection");
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading(if !app_state.connection.is_new {
            "Edit Connection"
        } else {
            "New Connection"
        });
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            ui.label("Connection Name:");
            ui.text_edit_singleline(&mut app_state.connection.name);
        });

        ui.horizontal(|ui| {
            ui.label("Database Type:");
            ui.radio_value(&mut app_state.connection.db_type, DbType::MySQL, "MySQL");
            ui.radio_value(
                &mut app_state.connection.db_type,
                DbType::PostgreSQL,
                "PostgreSQL",
            );
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

            if !app_state.connection.is_new {
                if ui.button("Remove Connection").clicked() {
                    app_state.connection.confirm_delete = true;
                }
            }
            if ui.button("Test Connection").clicked() {
                info!("Testing Connection");
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
                app_state.connection.success_message = None;
                app_state.connection.loading_message = Some("Testing connection...".to_string());

                let password = app_state.connection.password.clone();
                let (tx, rx) = tokio::sync::mpsc::channel(1);
                app_state.connection.rx = Some(rx);
                // This would be better with a callback mechanism
                app_state.runtime.spawn(async move {
                    if let Err(err) = db_manager
                        .connect(&test_connection, Some(password), true)
                        .await
                    {
                        tx.send(Err(err)).await.ok();
                    } else {
                        tx.send(Ok(())).await.ok();
                    }
                });
            }

            if ui.button("Save Connection").clicked() {
                app_state.connection.loading_message = None;
                app_state.connection.error_message = None;
                app_state.connection.success_message = None;
                if app_state.connection.name.trim().is_empty() {
                    app_state.connection.error_message =
                        Some("Connection name cannot be empty".to_string());
                } else if app_state.connection.database.trim().is_empty() {
                    app_state.connection.error_message =
                        Some("Database name cannot be empty".to_string());
                } else {
                    if let Err(err) = app_state.save_db() {
                        app_state.connection.error_message =
                            Some(format!("Failed to store password: {}", err));
                    } else {
                        app_state.connection.success_message =
                            Some("Connection saved successfully!".to_string());
                        app_state.mode = AppMode::Connections;
                    }
                }
            }
        });

        // Display error/success messages
        ui.add_space(10.0);
        if let Some(ref err) = app_state.connection.error_message {
            ui.colored_label(egui::Color32::RED, err);
        }

        if let Some(ref success) = app_state.connection.loading_message {
            ui.horizontal(|ui| {
                ui.spinner(); // Show a loading spinner
                ui.colored_label(egui::Color32::YELLOW, success); // Yellow for "in progress"
            });
        }
        if let Some(ref success) = app_state.connection.success_message {
            ui.colored_label(egui::Color32::GREEN, success);
        }

        if let Some(ref mut rx) = app_state.connection.rx {
            if let Ok(res) = rx.try_recv() {
                match res {
                    Ok(_) => {
                        app_state.connection.error_message = None;
                        app_state.connection.loading_message = None;
                        app_state.connection.success_message =
                            Some("Connection test successful".to_string());
                    }
                    Err(error_msg) => {
                        let res = format!("Connection test failed: {}", error_msg);

                        app_state.connection.loading_message = None;
                        app_state.connection.success_message = None;
                        app_state.connection.error_message = Some(res);
                    }
                }
            }
        }

        if app_state.connection.confirm_delete {
            Window::new("Confirm Save")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label("Are you sure you want to remove this connection?");

                    ui.horizontal(|ui| {
                        if ui.button("Yes").clicked() {
                            app_state.connection.confirm_delete = false;
                            if let Err(err) =
                                app_state.remove_db(app_state.connection.uuid.clone())
                            {
                                app_state.connection.loading_message = None;
                                app_state.connection.success_message = None;
                                app_state.connection.error_message =
                                    Some(format!("Failed to remove connection: {}", err));
                            } else {
                                app_state.mode = AppMode::Home;
                            }
                        }

                        if ui.button("No").clicked() {
                            app_state.connection.confirm_delete = false;
                        }
                    });
                });
        }
    });
}
