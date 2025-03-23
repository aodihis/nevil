use crate::app::{AppMode, AppState};
use crate::ui::connection::Connection;
use egui::{Align, Context, Layout};
use crate::security::SecureStorage;
use crate::ui::chat::Conversation;

pub fn left_panel_ui(ctx: &Context, app_state: &mut AppState) {
    egui::SidePanel::left("left_panel").resizable(true).show(ctx, |ui| {
        side_menu(ui, app_state);
        connection_list(ui, app_state);
    });
}

pub fn side_menu(ui: &mut egui::Ui, app_state: &mut AppState) {
    ui.horizontal(|ui| {
        ui.heading("neVil");
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if ui.add(egui::Button::new("⚙").frame(false)).clicked() {
                app_state.mode = AppMode::Settings;
            }
            if ui.add(egui::Button::new("➕").frame(false)).clicked() {
                app_state.mode = AppMode::Connections;
                app_state.connection = Connection::new();
            }
        });
    });
    ui.separator();
}

pub fn connection_list(ui: &mut egui::Ui, app_state: &mut AppState) {
    for con in &app_state.config.connections {
        ui.horizontal(|ui| {
            if ui.add(egui::Button::new(con.name.clone()).frame(false)).clicked() {
                app_state.conversation = Conversation::new(Some(con.uuid.clone()));
                app_state.mode = AppMode::Chat;
            }

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui.add(egui::Button::new("⚙").frame(false)).clicked() {
                    let mut existing_connection = Connection::new();
                    existing_connection.is_new = false;
                    existing_connection.uuid = con.uuid.clone();
                    existing_connection.name = con.name.clone();
                    existing_connection.database = con.database.clone();
                    existing_connection.host = con.host.clone();
                    existing_connection.port = con.port.clone();
                    existing_connection.db_type = con.db_type.clone();
                    existing_connection.username = con.username.clone();
                    if let Ok(pwd) = SecureStorage::get_db_password(&con.uuid.to_string()) {
                        existing_connection.password = pwd;
                    }
                    app_state.connection = existing_connection;
                    app_state.mode = AppMode::Connections;
                }
            });
        });

        ui.separator();
    }
}