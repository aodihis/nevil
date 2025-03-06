use crate::app::{AppMode, AppState};
use egui::{Align, Context, Layout};
use crate::config::{DbConnection, DbType};

pub fn left_panel(ctx: &Context, app_state: &mut AppState) {
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
        });
    });
    ui.separator();
}

pub fn connection_list(ui: &mut egui::Ui, app_state: &mut AppState) {
    for con in &app_state.config.connections {
        ui.horizontal(|ui| {
            ui.heading(con.name.clone());
        });
    }
}