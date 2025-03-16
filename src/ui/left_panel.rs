use crate::app::{AppMode, AppState};
use crate::ui::connection::Connection;
use egui::{Align, Context, Layout};
use crate::security::SecureStorage;

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
                app_state.mode = AppMode::Chat;
            }

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui.add(egui::Button::new("⚙").frame(false)).clicked() {
                    app_state.connection.is_new = false;
                    app_state.connection.uuid = con.uuid.clone();
                    app_state.connection.name = con.name.clone();
                    app_state.connection.database = con.database.clone();
                    app_state.connection.host = con.host.clone();
                    app_state.connection.port = con.port.clone();
                    app_state.connection.db_type = con.db_type.clone();
                    app_state.connection.username = con.username.clone();
                    if let Ok(pwd) = SecureStorage::get_db_password(&con.uuid.to_string()) {
                        app_state.connection.password = pwd;
                    }
                    app_state.connection.confirm_delete = false;
                    app_state.mode = AppMode::Connections;
                }
            });
        });

        ui.separator();
    }
}