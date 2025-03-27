use egui::Context;

use crate::app::{AppMode, AppState};
use crate::ui::chat::render_chat;
use crate::ui::connection::connection_ui;
use crate::ui::home::render_home;
use crate::ui::left_panel::left_panel_ui;
use crate::ui::query_result::render_result;
use crate::ui::setting::render_settings;

pub fn render_ui(ctx: &Context, app_state: &mut AppState) {
    left_panel_ui(ctx, app_state);

    match app_state.mode {
        AppMode::Home => render_home(),
        AppMode::Settings => render_settings(ctx, app_state),
        AppMode::Connections => connection_ui(ctx, app_state),
        AppMode::Chat => render_chat(ctx, app_state)
    }

    render_result(ctx, app_state);

}