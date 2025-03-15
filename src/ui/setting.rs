use crate::app::AppState;
use crate::llm::claude::Model as ClaudeModel;
use crate::llm::llm::Provider;
use crate::llm::openai::Model as OpenAiModel;
use egui::{Context, TextEdit};

#[derive(Clone)]
pub struct Settings {
    pub provider: Option<Provider>,
    pub model: String,
    pub api_key: String,
    pub success_message: Option<String>,
    pub error_message: Option<String>,
}


pub fn render_settings(ctx: &Context, app_state: &mut AppState) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("LLM API Settings");
            ui.add_space(10.0);

            let selected_provider = &mut app_state.settings.provider;

            let model_names = if let Some(provider) = selected_provider {
                match provider {
                    Provider::OpenAI => OpenAiModel::variants_name(),
                    Provider::Claude => ClaudeModel::variants_name(),
                }
            } else {
                vec![]
            };

            let provider = vec![Provider::Claude, Provider::OpenAI];

            ui.horizontal(|ui| {
                ui.label("Provider:");
                egui::ComboBox::new("provider", "")
                    .selected_text(selected_provider.as_ref().map(|p: &Provider| p.name()).unwrap_or("Choose..."))
                    .show_ui(ui, |ui| {
                        for p in provider {
                            if ui.selectable_value(selected_provider, Some(p.clone()), p.name()).clicked() {
                                if *selected_provider == Some(p.clone()) {
                                    app_state.settings.model = "".to_owned();
                                }
                            }
                        }
                    });
            });

            let model = &mut app_state.settings.model;
            ui.add_enabled_ui(selected_provider.is_some(), |ui| {
                ui.horizontal(|ui| {
                    ui.label("Model:");
                    egui::ComboBox::new("model", "")
                        .selected_text(model.clone())
                        .show_ui(ui, |ui| {
                            for item in model_names {
                                ui.selectable_value(model, item.parse().unwrap(), item);
                            }
                        })
                });
            });


            ui.horizontal(|ui| {
                ui.label("API Key:");
                ui.add(TextEdit::singleline(&mut app_state.settings.api_key).password(true));
            });

            if ui.button("Save API Settings").clicked() {
                app_state.save_settings();
            }

            // Display error/success messages
            if let Some(ref err) = app_state.settings.error_message {
                ui.colored_label(egui::Color32::RED, err);
            }

            if let Some(ref success) = app_state.settings.success_message {
                ui.colored_label(egui::Color32::GREEN, success);
            }
        });
    }

