use egui::{Context, TextEdit};
use crate::app::AppState;
use crate::llm::llm::Provider;

pub fn settings(ctx: &Context, app_state: &mut AppState){
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("LLM API Settings");
        ui.add_space(10.0);

        let  api_config = &mut app_state.config.llm_api.clone();
        let  selected_provider = &mut app_state.config.llm_api.provider;
        ui.horizontal(|ui| {
            ui.label("Provider:");
            egui::ComboBox::new("provider", "")
                .selected_text(selected_provider.as_ref().map(|p: &Provider| p.name()).unwrap_or("Choose..."))
                .show_ui(ui, |ui| {
                    ui.selectable_value(selected_provider, Some(Provider::OpenAI), Provider::OpenAI.name());
                    ui.selectable_value(selected_provider, Some(Provider::Claude), Provider::Claude.name());
            });

        });

        ui.horizontal(|ui| {
            ui.label("Model:");
            ui.text_edit_singleline(&mut api_config.model);
        });

        let mut api_key = app_state.config.llm_api.api_key.clone();

        ui.horizontal(|ui| {
            ui.label("API Key:");
            ui.add(TextEdit::singleline(&mut api_key).password(true));
        });

        if ui.button("Save API Settings").clicked() {

            // Save API key securely
            if !api_key.is_empty() {
                if let Err(err) = crate::security::SecureStorage::store_api_key(
                    &api_key
                ) {
                    app_state.error_message = Some(format!("Failed to store API key: {}", err));
                } else {
                    app_state.config.save();
                    app_state.error_message = None;
                    app_state.success_message = Some("API settings saved successfully!".to_string());
                }
            }
        }

        // Display error/success messages
        if let Some(ref err) = app_state.error_message {
            ui.colored_label(egui::Color32::RED, err);
        }

        if let Some(ref success) = app_state.success_message {
            ui.colored_label(egui::Color32::GREEN, success);
        }
    });
}