use egui::{Align, Context, Layout, RichText, ScrollArea, TextEdit};

use crate::app::{AppMode, AppState, MessageSender};
use crate::config::{DbConnection, DbType};
use crate::ui::connection::new_connection;
use crate::ui::left_panel::left_panel;
use crate::ui::setting::settings;

pub fn render_ui(ctx: &Context, app_state: &mut AppState) {
    left_panel(ctx,app_state);

    match app_state.mode {
        AppMode::Settings => settings(ctx, app_state),
        AppMode::Connections => render_connections(ctx, app_state),
        AppMode::Query => render_query(ctx, app_state),
        AppMode::NewConnection => new_connection(ctx, app_state)
    }
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