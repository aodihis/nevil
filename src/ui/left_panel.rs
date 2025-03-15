use crate::app::{AppMode, AppState};
use crate::ui::connection::Connection;
use egui::{Align, Context, Layout};

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
                app_state.mode = AppMode::NewConnection;
                app_state.connection = Connection::new();
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