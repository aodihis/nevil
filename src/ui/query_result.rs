use crate::app::AppState;
use crate::db_element::db::QueryResult;
use egui::Context;
use uuid::Uuid;

pub struct ResultTable {
    pub id: Uuid,
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


    if let Ok(res) = app_state.query_rx.try_recv() {
        match res {
            Ok(result) => {
                let index = app_state.conversation.loading_query.borrow().iter().position(|item| *item == result.id);
                if let Some(index) = index {
                    app_state.conversation.loading_query.borrow_mut().remove(index);
                }
                app_state.query_result.push(result);
            }
            Err(error_msg) => {
                let _ = format!("Connection test failed: {}", error_msg);
            }
        }
    }


}