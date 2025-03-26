use crate::app::AppState;
use crate::db_element::db::QueryResult;
use egui::Context;

pub struct ResultTable {
    pub id: u32,
    pub title: String,
    pub data: QueryResult,
}
pub fn render_result(ctx: &Context, app_state: &mut AppState) {
    app_state.query_result.retain(|window| {
        let mut open = true;
        egui::Window::new(&window.title)
            .open(&mut open)
            .show(ctx, |ui| {
                ui.label(&window.title);
            });
        open
    });
}