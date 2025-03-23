use chrono::Utc;
use crate::app::AppState;
use crate::db_element::chat::{Message, Sender};
use egui::{Align, Color32, Context, Frame, RichText, ScrollArea, TextEdit};
use uuid::Uuid;

pub struct Conversation {
    pub id: Option<Uuid>,
    pub message: Vec<Message>,
    message_input: String,
}

impl Conversation {
    pub fn new(uuid: Option<Uuid>) -> Self {
        Self { id: uuid, message: Vec::new(), message_input: "".to_string() }
    }
}
pub fn render_chat(ctx: &Context, app_state: &mut AppState) {

    egui::CentralPanel::default().show(ctx, |ui| {
        // Chat area
        let available_height = ui.available_height();
        let chat_height = available_height * 0.9;

        ScrollArea::vertical()
            .auto_shrink([false; 2])
            .stick_to_bottom(true)
            .max_height(chat_height)
            .show(ui, |ui| {
                for msg in &app_state.conversation.message {
                    let (align, bubble_color) = match msg.sender {
                        Sender::User => (egui::Layout::right_to_left(Align::RIGHT), Color32::from_rgb(0, 150, 255)),
                        Sender::System => (egui::Layout::left_to_right(Align::RIGHT), Color32::from_rgb(230, 230, 230)),
                    };

                    ui.with_layout(align, |ui| {
                        Frame::NONE
                            .fill(bubble_color)
                            .corner_radius(egui::CornerRadius::same(12))
                            .inner_margin(egui::Margin::symmetric(10, 8))
                            .show(ui, |ui| {
                                let text_color = if let Sender::User = msg.sender {
                                    Color32::WHITE
                                } else {
                                    Color32::BLACK
                                };
                                ui.colored_label(text_color, &msg.content);
                            });
                    });

                    ui.add_space(5.0);



                }
            });

        // Input area
        ui.separator();
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            let input = TextEdit::multiline(&mut app_state.conversation.message_input)
                .frame(true)
                .desired_width(ui.available_width() - 100.0)
                .desired_rows(2)
                .hint_text("Ask a question about your data...");

            let _ = ui.add(input);
            let send_clicked = ui.button("Send").clicked();
            let enter_pressed = ui.input(|i| i.key_pressed(egui::Key::Enter) && !i.modifiers.shift);

            // Send button
            if send_clicked || enter_pressed {
                if !app_state.conversation.message_input.trim().is_empty() {
                    let uuid = app_state.conversation.id.unwrap();
                    if let Ok(msg) = app_state.send_message(&uuid, app_state.conversation.message_input.clone()) {
                        app_state.conversation.message.push(msg);
                        app_state.conversation.message_input.clear();

                        app_state.conversation.message.push(Message {
                            uuid: Uuid::new_v4(),
                            sender: Sender::System,
                            content: "System Message".to_string(),
                            is_sql: false,
                            timestamp: Utc::now(),
                        });
                    }
                }
            }
        });
    });
}