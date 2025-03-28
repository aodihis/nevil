use crate::app::AppState;
use crate::db_element::db::QueryResult;
use eframe::emath::Align;
use egui::{Color32, Context, Frame, RichText, TextEdit, Ui, Window};
use egui_extras::{Column, Size, StripBuilder, TableBuilder};
use uuid::Uuid;

pub struct ResultTable {
    pub id: Uuid,
    pub connection_id: Uuid,
    pub query: String,
    pub data: QueryResult,
    pub is_open: bool,
    pub edited_page: usize,
}
pub fn render_result(ctx: &Context, app_state: &mut AppState) {

    for i in 0..app_state.query_result.len() {
    // for window in &mut app_state.query_result {
        let width = ctx.screen_rect().width() * 0.9;
        let height = ctx.screen_rect().height() * 0.85;

        let mut is_open = app_state.query_result[i].is_open;

        Window::new(&app_state.query_result[i].query).open(&mut is_open).default_width(width).show(ctx, |ui| {
            Frame::NONE.show(ui, |ui| {
                StripBuilder::new(ui).size(Size::exact(10.0)).size(Size::initial(height))
                    .vertical(|mut strip| {
                        strip.cell(|ui| {
                            if app_state.query_result[i].data.total_pages > 1 {
                                ui.add_space(5.0);
                                render_pagination(ui, app_state, i);
                                ui.add_space(5.0);
                                ui.separator();
                            }
                        });
                        strip.cell(|ui| {
                            ui.vertical_centered(|ui| {
                                egui::ScrollArea::horizontal()
                                    .show(ui, |ui| {
                                        render_table(&app_state.query_result[i], ui);
                                });
                                ui.separator();
                            });
                        });
                    });
            });
        });
        app_state.query_result[i].is_open = is_open;
    }

    app_state.query_result.retain(|r| r.is_open);

    if let Ok(res) = app_state.query_rx.try_recv() {
        match res {
            Ok(result) => {
                let index = app_state.conversation.loading_query.borrow().iter().position(|item| *item == result.id);
                if let Some(index) = index {
                    app_state.conversation.loading_query.borrow_mut().remove(index);
                }

                let index = app_state.query_result.iter().position(|r| r.id == result.id);
                if let Some(index) = index {
                    app_state.query_result[index] = result;
                } else {
                    app_state.query_result.push(result);
                }

            }
            Err(error_msg) => {
                let _ = format!("Connection test failed: {}", error_msg);
            }
        }
    }


}

fn render_pagination(ui: &mut Ui, app_state: &mut AppState, index: usize) {
    ui.horizontal(|ui| {
        if app_state.query_result[index].data.current_page > 1 {
            if ui.button("Prev").clicked() {
                app_state.run_query(&app_state.query_result[index].connection_id,
                                    &app_state.query_result[index].query,
                                    &app_state.query_result[index].id, app_state.query_result[index].data.current_page - 1);
            }
        }

        let mut page_str = app_state.query_result[index].edited_page.to_string();
        let response = ui.add_sized([20.0, 20.0], TextEdit::singleline(&mut page_str));
        if let Ok(current_page) = page_str.parse::<usize>() {
            app_state.query_result[index].edited_page = current_page;
        }

        if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter) && !i.modifiers.shift) {
            if app_state.query_result[index].edited_page != app_state.query_result[index].data.current_page {
                app_state.run_query(&app_state.query_result[index].connection_id,
                                    &app_state.query_result[index].query,
                                    &app_state.query_result[index].id, app_state.query_result[index].edited_page);
            }
        }

        ui.label(format!("/ {}", app_state.query_result[index].data.total_pages));
        if app_state.query_result[index].data.current_page < app_state.query_result[index].data.total_pages {
            if ui.button("Next").clicked() {
                app_state.run_query(&app_state.query_result[index].connection_id,
                                    &app_state.query_result[index].query,
                                    &app_state.query_result[index].id, app_state.query_result[index].data.current_page + 1);
            }
        }
    });
}
fn render_table(window: &ResultTable, ui: &mut Ui) {
    let mut table = TableBuilder::new(ui)
        .striped(true)
        .resizable(true)
        .cell_layout(egui::Layout::left_to_right(Align::LEFT));


    let bg = Color32::LIGHT_GRAY;
    let paint_bg = |ui: &mut Ui| {
        let item_spacing = ui.spacing().item_spacing;
        let gapless_rect = ui.max_rect().expand2(0.5 * item_spacing);
        ui.painter().rect_filled(gapless_rect, 0.0, bg);
    };

    let mut sizes = vec![];

    for col in &window.data.columns {
        sizes.push(col.len());
    }

    for row in &window.data.rows {
        let len = row.len();
        for i in 0..len {
            sizes[i] = row[i].len().max(sizes[i]);
        }
    }

    for i in 0..window.data.columns.len() {
        table = table.column(
            if sizes[i] > 50 {
                Column::initial(sizes[i].min(150) as f32).clip(true)
            } else {
                Column::auto()
            }
        );
    }
    table
        .header(20.0, |mut header| {
            for col in &window.data.columns {
                header.col(|ui| {
                    paint_bg(ui);
                    ui.vertical(|ui| {
                        ui.label(RichText::new(col).strong().color(Color32::BLACK));
                        ui.painter().hline(
                            ui.available_rect_before_wrap().x_range(),
                            ui.available_rect_before_wrap().bottom(),
                            egui::Stroke::new(1.0, Color32::DARK_GRAY)
                        );
                    });

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
        });

}