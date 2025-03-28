use crate::app::{AppMode, AppState};
use crate::security::SecureStorage;
use crate::ui::chat::Conversation;
use crate::ui::connection::Connection;
use egui::{Align, Context, Layout};
use log::info;

pub fn left_panel_ui(ctx: &Context, app_state: &mut AppState) {
    info!("Rendering left panel");
    egui::SidePanel::left("left_panel")
        .resizable(true)
        .show(ctx, |ui| {
            side_menu(ui, app_state);
            connection_list(ui, app_state);
        });
}

pub fn side_menu(ui: &mut egui::Ui, app_state: &mut AppState) {
    info!("Rendering side menu");
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
    info!("Rendering connections left panel");
    for con in &app_state.config.connections {
        ui.horizontal(|ui| {
            if ui
                .add(egui::Button::new(con.name.clone()).frame(false))
                .clicked()
            {
                app_state.conversation = Conversation::new(Some(con.uuid.clone()));
                app_state.conversation.messages = app_state
                    .chat_storage
                    .get_conversation(&con.uuid)
                    .unwrap_or_else(|_| vec![]);
                let db_config = con.clone();
                let pass = if let Ok(pwd) = SecureStorage::get_db_password(&con.uuid.to_string()) {
                    pwd
                } else {
                    return;
                };
                let db_manager = app_state.db_manager.clone();
                app_state
                    .runtime
                    .spawn(async move { db_manager.connect(&db_config, Some(pass), false).await });
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
