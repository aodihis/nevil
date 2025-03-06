use egui::{Context, TextEdit, Window};
use crate::app::{AppMode, AppState};
use crate::config::DbType;

pub fn new_connection(ctx: &Context, app_state: &mut AppState) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading(if app_state.editing_connection { "Edit Connection" } else { "New Connection" });
        ui.add_space(10.0);

        if let Some(ref mut connection) = app_state.new_connection.clone() {
            ui.horizontal(|ui| {
                ui.label("Connection Name:");
                ui.text_edit_singleline(&mut connection.name);
            });

            ui.horizontal(|ui| {
                ui.label("Database Type:");
                ui.radio_value(&mut connection.db_type, DbType::MySQL, "MySQL");
                ui.radio_value(&mut connection.db_type, DbType::PostgreSQL, "PostgreSQL");
            });

            // Update connection string template when DB type changes
            match connection.db_type {
                DbType::MySQL => {
                    connection.connection_string_template = "mysql://{username}:{password}@{host}:{port}/{database}".to_string();
                    if connection.port == 5432 {
                        connection.port = 3306;
                    }
                },
                DbType::PostgreSQL => {
                    connection.connection_string_template = "postgres://{username}:{password}@{host}:{port}/{database}".to_string();
                    if connection.port == 3306 {
                        connection.port = 5432;
                    }
                },
            }

            ui.horizontal(|ui| {
                ui.label("Host:");
                ui.text_edit_singleline(&mut connection.host);
            });

            ui.horizontal(|ui| {
                ui.label("Port:");
                let mut port_str = connection.port.to_string();
                ui.text_edit_singleline(&mut port_str);
                if let Ok(port) = port_str.parse::<u16>() {
                    connection.port = port;
                }
            });

            ui.horizontal(|ui| {
                ui.label("Database Name:");
                ui.text_edit_singleline(&mut connection.database);
            });

            ui.horizontal(|ui| {
                ui.label("Username:");
                ui.text_edit_singleline(&mut connection.username);
            });

            if let Some(ref mut password) = app_state.new_connection_password {
                ui.horizontal(|ui| {
                    ui.label("Password:");
                    ui.add(TextEdit::singleline(password).password(true));
                });
            }

            ui.add_space(20.0);

            ui.horizontal(|ui| {
                if ui.button("Cancel").clicked() {
                    app_state.mode = AppMode::Connections;
                    app_state.new_connection = None;
                    app_state.new_connection_password = None;
                    app_state.editing_connection = false;
                }

                if ui.button("Test Connection").clicked() {
                    if let Some(ref password) = app_state.new_connection_password {
                        // Store password temporarily
                        if let Err(err) = crate::security::SecureStorage::store_db_password(
                            &connection.name,
                            password
                        ) {
                            app_state.error_message = Some(format!("Failed to securely store password: {}", err));
                        } else {
                            // Create a clone of connection for testing
                            let test_connection = connection.clone();
                            let db_manager = app_state.db_manager.clone();

                            // Test connection asynchronously
                            app_state.error_message = None;
                            app_state.success_message = Some("Testing connection...".to_string());

                            let ctx = ctx.clone();
                            // This would be better with a callback mechanism
                            app_state.runtime.spawn(async move {
                                if let Err(err) = db_manager.connect(&test_connection).await {
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
                    }
                }

                if ui.button("Save Connection").clicked() {
                    if connection.name.trim().is_empty() {
                        app_state.error_message = Some("Connection name cannot be empty".to_string());
                    } else if connection.database.trim().is_empty() {
                        app_state.error_message = Some("Database name cannot be empty".to_string());
                    } else if let Some(ref password) = app_state.new_connection_password {
                        if !app_state.editing_connection &&
                            app_state.config.connections.iter().any(|c| c.name == connection.name) {
                            app_state.error_message = Some("A connection with this name already exists".to_string());
                        } else {
                            // Store password securely
                            if let Err(err) = crate::security::SecureStorage::store_db_password(
                                &connection.name,
                                password
                            ) {
                                app_state.error_message = Some(format!("Failed to store password: {}", err));
                            } else {
                                if app_state.editing_connection {
                                    // Update existing connection
                                    if let Some(idx) = app_state.config.connections.iter().position(|c| c.name == connection.name) {
                                        app_state.config.connections[idx] = connection.clone();
                                    } else {
                                        app_state.config.connections.push(connection.clone());
                                    }
                                } else {
                                    // Add new connection
                                    app_state.config.connections.push(connection.clone());
                                }

                                // Save the config
                                app_state.config.save();
                                app_state.success_message = Some("Connection saved successfully!".to_string());
                                app_state.error_message = None;
                                app_state.mode = AppMode::Connections;
                                app_state.new_connection = None;
                                app_state.new_connection_password = None;
                                app_state.editing_connection = false;
                            }
                        }
                    }
                }
            });

            // Display error/success messages
            ui.add_space(10.0);
            if let Some(ref err) = app_state.error_message {
                ui.colored_label(egui::Color32::RED, err);
            }

            if let Some(ref success) = app_state.success_message {
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
                    app_state.success_message = Some("AI-assisted setup would launch here".to_string());

                    // In a complete implementation, this would:
                    // 1. Launch a dialog to ask the user for high-level info about their database
                    // 2. Use the LLM to generate connection parameters
                    // 3. Pre-fill the form with these parameters
                }
            }
        }
    });
}