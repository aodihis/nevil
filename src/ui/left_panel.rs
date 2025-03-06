use crate::app::{AppMode, AppState};
use egui::{Align, Context, Layout};

pub fn left_panel(ctx: &Context, app_state: &mut AppState) {
    egui::SidePanel::left("left_panel").resizable(true).show(ctx, |ui| {
        side_menu(ui, app_state);
    });
}

pub fn side_menu(ui: &mut egui::Ui, app_state: &mut AppState) {
    ui.horizontal(|ui| {
        ui.heading("Nevil");
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if ui.add(egui::Button::new("⚙").frame(false)).clicked() {
                app_state.mode = AppMode::Settings;
            }
        });
    });
    ui.separator();
}