use eframe::emath::Align;
use crate::app::AppState;
use crate::db_element::db::QueryResult;
use egui::{Context, RichText};
use uuid::Uuid;
use egui_extras::{TableBuilder, Column};

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

                let mut table = TableBuilder::new(ui)
                    .striped(true)
                    .resizable(true)
                    .cell_layout(egui::Layout::left_to_right(Align::Center));

                for _ in 0..window.data.columns.len() {
                    table = table.column(Column::remainder());
                }

                table
                    .header(20.0, |mut header| {
                        for col in &window.data.columns {
                            header.col(|ui| {
                               ui.strong(&col.to_string());
                            });
                        }
                    })
                    .body(|mut body| {
                        for data_row in &window.data.rows {
                            body.row(20.0, |mut row| {
                                for cell in data_row {
                                    row.col(|ui| {
                                        ui.label(RichText::new(cell.to_string()));
                                    });
                                }
                            })

                        }
                    })
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