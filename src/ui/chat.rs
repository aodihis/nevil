use eframe::emath::Align;
use egui::{Context, Layout, RichText, ScrollArea, TextEdit};
use crate::app::{AppState, MessageSender};

pub fn render_chat(ctx: &Context, app_state: &mut AppState) {

    egui::CentralPanel::default().show(ctx, |ui| {
        // Chat area
        let available_height = ui.available_height();
        let chat_height = available_height * 0.6;
        let result_height = available_height * 0.4;

        ScrollArea::vertical()
            .auto_shrink([false; 2])
            .stick_to_bottom(true)
            .max_height(chat_height)
            .show(ui, |ui| {
                for message in &app_state.chat_messages {
                    match message.sender {
                        MessageSender::User => {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("You:").strong());
                                ui.label(&message.content);
                            });
                        },
                        MessageSender::Assistant => {
                            if message.is_sql {
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new("SQL Query:").strong().color(egui::Color32::from_rgb(0, 128, 0)));
                                    ui.monospace(&message.content);
                                });
                            } else {
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new("Assistant:").strong().color(egui::Color32::from_rgb(0, 0, 128)));
                                    ui.label(&message.content);
                                });
                            }
                        },
                        MessageSender::System => {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("System:").strong().color(egui::Color32::from_rgb(128, 0, 0)));
                                ui.label(&message.content);
                            });
                        }
                    }
                    ui.add_space(4.0);
                }
            });

        // Input area
        ui.separator();
        ui.horizontal(|ui| {
            let text_edit = TextEdit::multiline(&mut app_state.current_message)
                .desired_width(ui.available_width() - 100.0)
                .desired_rows(2)
                .hint_text("Ask a question about your data...");

            ui.add(text_edit);

            if ui.button("Send").clicked() {
                // app_state.send_message();
            }
        });
    });
}