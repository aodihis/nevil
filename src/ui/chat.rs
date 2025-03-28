use std::cell::RefCell;
use crate::app::{AppMode, AppState};
use crate::db_element::chat::{Message, Sender};
use crate::db_element::db::DatabaseManager;
use crate::llm::llm::{LLMClient, ResponseType};
use egui::{Align, Color32, Context, Frame, ScrollArea, TextEdit};
use log::debug;
use uuid::Uuid;

pub struct Conversation {
    pub id: Option<Uuid>,
    pub messages: Vec<Message>,
    is_loading: bool,
    pub loading_query: RefCell<Vec<Uuid>>,
    message_input: String,
    rx: Option<tokio::sync::mpsc::Receiver<Result<(Message,Vec<Message>), String>>>
}

impl Conversation {
    pub fn new(uuid: Option<Uuid>) -> Self {
        Self { id: uuid, messages: Vec::new(), is_loading: false, loading_query: RefCell::new(vec![]), message_input: "".to_string(), rx: None }
    }
}
pub fn render_chat(ctx: &Context, app_state: &mut AppState) {
    egui::CentralPanel::default().show(ctx, |ui| {
        let uuid = app_state.conversation.id.unwrap();
        let llm_client = match &app_state.llm_client {
            Some(client) => client.clone(),
            _ => {
                app_state.mode = AppMode::Home;
                return;
            }
        };
        // Chat area
        let available_height = ui.available_height();
        let chat_height = available_height * 0.85;

        ScrollArea::vertical()
            .auto_shrink([false; 2])
            .stick_to_bottom(true)
            .max_height(chat_height)
            .show(ui, |ui| {
                for msg in &app_state.conversation.messages {

                    let (align, bubble_color, valign) = match msg.sender {
                        Sender::User => (egui::Layout::right_to_left(Align::RIGHT), Color32::from_rgb(0, 150, 255), Align::RIGHT),
                        Sender::System => (egui::Layout::left_to_right(Align::RIGHT), Color32::from_rgb(230, 230, 230), Align::LEFT),
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

                                if msg.is_sql {
                                    ui.with_layout(egui::Layout::top_down(valign), |ui| {
                                        ui.horizontal_wrapped(|ui| {
                                            ui.colored_label(text_color, &msg.content);
                                        });
                                        if app_state.conversation.loading_query.borrow().contains(&msg.uuid) {
                                            ui.add_enabled(false, egui::Button::new("⏳ Running..."));
                                        } else {
                                            if ui.button("▶ Run Query").clicked() {
                                                app_state.conversation.loading_query.borrow_mut().push(msg.uuid);
                                                app_state.run_query(&uuid, &msg.content, &msg.uuid);
                                            }
                                        }
                                    });
                                } else {
                                    ui.horizontal_wrapped(|ui| {
                                        ui.colored_label(text_color, &msg.content);
                                    });
                                }
                            });
                    });

                    ui.add_space(5.0);
                }
            });

        // Input area
        ui.separator();
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            let mut response = None;
                ScrollArea::vertical()
                .max_height(ui.available_height())
                .show(ui, |ui| {
                    let input = TextEdit::multiline(&mut app_state.conversation.message_input)
                        .frame(true)
                        .desired_width(ui.available_width() - 100.0)
                        .desired_rows(4)
                        .hint_text("Ask a question about your data...");
                    response = Some(ui.add(input));
                });
            // ui.add_space(4.0);
            if app_state.conversation.is_loading {
                ui.add_enabled(false, egui::Button::new("⏳ Running..."));
                return;
            }
            let send_clicked = ui.button("Send").clicked();
            let enter_pressed =  if let Some(response) = response {
                response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter) && !i.modifiers.shift)
            } else {
                false
            };

            // Send button
            if send_clicked || enter_pressed {
                if !app_state.conversation.message_input.trim().is_empty() {
                    let uuid = uuid.clone();
                    let (tx, rx) = tokio::sync::mpsc::channel(1);
                    app_state.conversation.is_loading = true;
                    app_state.conversation.rx = Some(rx);
                    let db_manager = app_state.db_manager.clone();
                    let message = app_state.conversation.message_input.clone();
                    app_state.conversation.message_input.clear();
                    let llm_client = llm_client.clone();
                    app_state.runtime.spawn(async move {
                        let res = send_message(&llm_client, &db_manager, &uuid, message).await;
                       tx.send(res).await.ok();
                    });

                }
            }
        });

        if let Some(ref mut rx) = app_state.conversation.rx {
            if let Ok(recv) = rx.try_recv() {
                if let Ok(res) = recv {
                    let (user_message, system_messages) = res;
                    app_state.chat_storage.add_message(&uuid, &user_message).expect("Failed to add message");
                    app_state.conversation.messages.push(user_message);
                    system_messages.into_iter().for_each(|system_message| {
                        app_state.chat_storage.add_message(&uuid, &system_message).expect("Failed to add message");
                        app_state.conversation.messages.push(system_message);
                    });
                    app_state.conversation.is_loading = false;
                }
            }
        }



    });
}


pub async fn send_message(llm_client:  &LLMClient, db_manager: &DatabaseManager, element_uuid: &Uuid, msg: String) -> Result<(Message, Vec<Message>), String> {
    let message = Message::new(Sender::User, msg, false);
    let schema = db_manager.get_schema_info(element_uuid).await?;
    let response = match llm_client.generate_sql(&message.content, &schema).await {
        Ok(res) => {
            debug!("System response: {:?}", res);
            res
        }
        Err(e) => {
            debug!("Error generating SQL statement: {:?}", e);
            return Err("Failed to communicate with LLM".to_string());
        }
    };

    // let system_response = Message::new(Sender::System, response.message, response.r#type == ResponseType::Query);
    let system_responses = response.iter().map(|res| {
        Message::new(Sender::System, res.message.to_string(), res.r#type == ResponseType::Query)
    }).collect();
    Ok((message, system_responses))
}