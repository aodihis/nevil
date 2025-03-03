use egui::{Align, Context, Layout, RichText, ScrollArea, TextEdit};

use crate::app::{AppMode, AppState, MessageSender};
use crate::config::{DbConnection, DbType};

pub fn render_ui(ctx: &Context, app_state: &mut AppState) {
    egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.heading("Database Query Assistant");
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui.button("Settings").clicked() {
                    app_state.mode = AppMode::Settings;
                }
                if ui.button("Connections").clicked() {
                    app_state.mode = AppMode::Connections;
                }
                if ui.button("Query").clicked() {
                    app_state.mode = AppMode::Query;
                }
            });
        });
    });

    match app_state.mode {
        AppMode::Settings => render_settings(ctx, app_state),
        AppMode::Connections => render_connections(ctx, app_state),
        AppMode::Query => render_query(ctx, app_state),
        AppMode::NewConnection => render_new_connection(ctx, app_state),
    }
}

fn render_settings(ctx: &Context, app_state: &mut AppState) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("LLM API Settings");
        ui.add_space(10.0);

        let mut api_config = app_state.config.llm_api.clone();

        ui.horizontal(|ui| {
            ui.label("Provider:");
            ui.text_edit_singleline(&mut api_config.provider);
        });

        ui.horizontal(|ui| {
            ui.label("Model:");
            ui.text_edit_singleline(&mut api_config.model);
        });

        let mut api_key = match app_state.stored_api_key.clone() {
            Some(key) => key,
            None => "".to_string(),
        };

        ui.horizontal(|ui| {
            ui.label("API Key:");
            ui.add(TextEdit::singleline(&mut api_key).password(true));
        });

        if ui.button("Save API Settings").clicked() {
            app_state.config.llm_api = api_config;
            app_state.stored_api_key = Some(api_key.clone());

            // Save API key securely
            if !api_key.is_empty() {
                if let Err(err) = crate::security::SecureStorage::store_api_key(
                    &app_state.config.llm_api.provider,
                    &api_key
                ) {
                    app_state.error_message = Some(format!("Failed to store API key: {}", err));
                } else {
                    app_state.config.save();
                    app_state.error_message = None;
                    app_state.success_message = Some("API settings saved successfully!".to_string());
                }
            }
        }

        // Display error/success messages
        if let Some(ref err) = app_state.error_message {
            ui.colored_label(egui::Color32::RED, err);
        }

        if let Some(ref success) = app_state.success_message {
            ui.colored_label(egui::Color32::GREEN, success);
        }
    });
}

fn render_connections(ctx: &Context, app_state: &mut AppState) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("Database Connections");
        ui.add_space(10.0);

        if ui.button("Add New Connection").clicked() {
            app_state.mode = AppMode::NewConnection;
            app_state.new_connection = Some(DbConnection {
                name: "New Connection".to_string(),
                db_type: DbType::MySQL,
                host: "localhost".to_string(),
                port: 3306,
                username: "root".to_string(),
                database: "".to_string(),
                connection_string_template: "mysql://{username}:{password}@{host}:{port}/{database}".to_string(),
            });
            app_state.new_connection_password = Some("".to_string());
        }

        ui.add_space(20.0);

        ScrollArea::vertical().show(ui, |ui| {
            for connection in app_state.config.connections.clone() {

                ui.horizontal(|ui| {
                    ui.label(RichText::new(&connection.name).size(16.0));
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui.button("Delete").clicked() {
                            if let Some(idx) = app_state.config.connections.iter().position(|c| c.name == connection.name) {
                                app_state.config.connections.remove(idx);
                                app_state.config.save();
                            }
                        }

                        if ui.button("Test").clicked() {
                            app_state.test_connection(connection.name.clone());
                        }

                        if ui.button("Edit").clicked() {
                            app_state.mode = AppMode::NewConnection;
                            app_state.new_connection = Some(connection.clone());
                            app_state.new_connection_password = Some("".to_string());
                            app_state.editing_connection = true;
                        }
                    });
                });

                ui.horizontal(|ui| {
                    ui.label(format!("Type: {}", match connection.db_type {
                        DbType::MySQL => "MySQL",
                        DbType::PostgreSQL => "PostgreSQL",
                    }));
                    ui.label(format!("Host: {}:{}", connection.host, connection.port));
                    ui.label(format!("Database: {}", connection.database));
                });

                ui.separator();
            }
        });

        // Display error/success messages
        if let Some(ref err) = app_state.error_message {
            ui.colored_label(egui::Color32::RED, err);
        }

        if let Some(ref success) = app_state.success_message {
            ui.colored_label(egui::Color32::GREEN, success);
        }
    });
}

fn render_new_connection(ctx: &Context, app_state: &mut AppState) {
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

                            // This would be better with a callback mechanism
                            app_state.runtime.spawn(async move {
                                if let Err(err) = db_manager.connect(&test_connection).await {
                                    eprintln!("Connection test failed: {}", err);
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

fn render_query(ctx: &Context, app_state: &mut AppState) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.heading("Database Query");

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                egui::ComboBox::from_label("Connection")
                    .selected_text(match &app_state.active_connection {
                        Some(conn) => conn,
                        None => "Select connection",
                    })
                    .show_ui(ui, |ui| {
                        for connection in &app_state.config.connections {
                            ui.selectable_value(
                                &mut app_state.active_connection,
                                Some(connection.name.clone()),
                                &connection.name
                            );
                        }
                    });
            });
        });

        ui.separator();

        // Chat area
        let available_height = ui.available_height();
        let chat_height = available_height * 0.6;
        let result_height = available_height * 0.4;

        ScrollArea::vertical()
            .auto_shrink([false; 2])
            .stick_to_bottom(true)
            .max_height(chat_height)
            .show(ui, |ui| {
                for message in &app_state.chat_messages {
                    match message.sender {
                        MessageSender::User => {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("You:").strong());
                                ui.label(&message.content);
                            });
                        },
                        MessageSender::Assistant => {
                            if message.is_sql {
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new("SQL Query:").strong().color(egui::Color32::from_rgb(0, 128, 0)));
                                    ui.monospace(&message.content);
                                });
                            } else {
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new("Assistant:").strong().color(egui::Color32::from_rgb(0, 0, 128)));
                                    ui.label(&message.content);
                                });
                            }
                        },
                        MessageSender::System => {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("System:").strong().color(egui::Color32::from_rgb(128, 0, 0)));
                                ui.label(&message.content);
                            });
                        }
                    }
                    ui.add_space(4.0);
                }
            });

        // Query results table
        if let Some(ref result) = app_state.query_result {
            ui.separator();
            ui.heading("Query Results");

            ScrollArea::both()
                .max_height(result_height)
                .show(ui, |ui| {
                    egui::Grid::new("results_grid")
                        .striped(true)
                        .spacing([10.0, 4.0])
                        .show(ui, |ui| {
                            // Header row
                            for column in &result.columns {
                                ui.label(RichText::new(column).strong());
                            }
                            ui.end_row();

                            // Data rows
                            for row in &result.rows {
                                for cell in row {
                                    ui.label(cell);
                                }
                                ui.end_row();
                            }
                        });
                });
        }

        // Input area
        ui.separator();
        ui.horizontal(|ui| {
            let text_edit = TextEdit::multiline(&mut app_state.current_message)
                .desired_width(ui.available_width() - 100.0)
                .desired_rows(2)
                .hint_text("Ask a question about your data...");

            ui.add(text_edit);

            if ui.button("Send").clicked() {
                app_state.send_message();
            }
        });
    });
}