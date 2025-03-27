use crate::app::AppState;
use crate::db_element::db::QueryResult;
use eframe::emath::Align;
use egui::{Color32, Context, Frame, RichText, Ui, Window};
use egui_extras::{Column, Size, StripBuilder, TableBuilder};
use uuid::Uuid;

pub struct ResultTable {
    pub id: Uuid,
    pub title: String,
    pub data: QueryResult,
}
pub fn render_result(ctx: &Context, app_state: &mut AppState) {
    app_state.query_result.retain(|window| {
        let mut open = true;
        let width = ctx.screen_rect().width() * 0.9;
        let height = ctx.screen_rect().height() * 0.85;
        Window::new(&window.title)
            .open(&mut open)
            .default_width(width)
            .show(ctx, |ui| {
                Frame::NONE
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            StripBuilder::new(ui)
                                .size(Size::exact(1.0))
                                .size(Size::initial(height))
                                .vertical(|mut strip| {
                                    strip.cell(|ui| {
                                        ui.add_space(4.0);
                                    });
                                    strip.cell(|ui| {
                                        ui.vertical_centered(|ui| {
                                            egui::ScrollArea::horizontal()
                                                .show(ui, |ui| {
                                                    render_table(window, ui);
                                            });
                                        });
                                    });
                                });
                        });

                    });


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

fn render_table(window: &ResultTable, ui: &mut Ui) {
    let mut table = TableBuilder::new(ui)
        .striped(true)
        .resizable(true)
        .cell_layout(egui::Layout::left_to_right(Align::Center));

    for _ in 0..window.data.columns.len() {
        table = table.column(Column::auto());
    }


    let bg = Color32::LIGHT_GRAY;
    let paint_bg = |ui: &mut egui::Ui| {
        let item_spacing = ui.spacing().item_spacing;
        let gapless_rect = ui.max_rect().expand2(0.5 * item_spacing);
        ui.painter().rect_filled(gapless_rect, 0.0, bg);
    };

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