use crate::config::{AppConfig, DbConnection};
use crate::llm::llm::LLMClient;
use crate::security::SecureStorage;
use crate::ui::connection::Connection;
use crate::ui::setting::Settings;
use crate::ui::ui::render_ui;
use eframe::egui;
use std::sync::Arc;
use tokio::runtime::Runtime;
use uuid::Uuid;
use crate::db_element::db::{DatabaseManager, QueryResult};

pub enum AppMode {
    Home,
    Chat,
    Connections,
    Settings,
}

pub struct AppState {
    pub config: AppConfig,

    pub mode: AppMode,
    pub db_manager: Arc<DatabaseManager>,
    pub llm_client: Option<LLMClient>,
    pub active_connection: Option<String>,
    pub chat_messages: Vec<ChatMessage>,
    pub current_message: String,
    pub query_result: Option<QueryResult>,

    pub runtime: Runtime,

    // UI related struct
    pub settings: Settings,
    pub connection: Connection,
}

pub struct ChatMessage {
    pub sender: MessageSender,
    pub content: String,
    pub is_sql: bool,
}

pub enum MessageSender {
    User,
    Assistant,
    System,
}

pub struct DBQueryApp {
    state: AppState,
}

impl DBQueryApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Set up custom fonts
        let fonts = egui::FontDefinitions::default();
        // You could add custom fonts here if needed
        cc.egui_ctx.set_fonts(fonts);

        // Load configuration
        let config = AppConfig::load();

        // Create runtime for async operations
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed to create Tokio runtime");

        // Create LLM client if API key is available
        let llm_client = Some(LLMClient::new(config.llm_api.clone()));

        // Create database manager
        let db_manager = Arc::new(DatabaseManager::new());

        let provider = config.llm_api.provider.clone();
        let model = config.llm_api.model.clone();
        let api_key = match SecureStorage::get_api_key() {
            Ok(key) => {key}
            Err(_) => {"".to_string()}
        };
        Self {
            state: AppState {
                config,
                settings: Settings{
                    provider,
                    model,
                    api_key,
                    success_message: None,
                    error_message: None,
                },
                mode: AppMode::Home,
                db_manager,
                llm_client,
                active_connection: None,
                chat_messages: Vec::new(),
                current_message: String::new(),
                query_result: None,
                connection: Connection::new(),
                runtime,
            },
        }
    }
}

impl AppState {
    pub fn save_connection(&mut self) -> Result<(), String> {
        let connection = self.connection.clone();
        let password = connection.password;
        if let Err(err) = SecureStorage::store_db_password(&connection.uuid.to_string(), &password) {
            return Err(err.to_string());
        }

        let db_connection = DbConnection {
            uuid: connection.uuid,
            name: connection.name,
            db_type: connection.db_type,
            host: connection.host,
            port: connection.port,
            username: connection.username,
            database: connection.database,
        };
        // Update or add the connection
        if !connection.is_new  {
            if let Some(idx) = self.config.connections.iter().position(|c| c.uuid == connection.uuid) {
                self.config.connections[idx] = db_connection;
            } else {
                self.config.connections.push(db_connection);
            }
        } else {
            self.config.connections.push(db_connection);
        }

        // Save the config
        self.config.save();
        Ok(())
    }

    pub fn remove_connection(&mut self, connection_uuid: Uuid) -> Result<(), String>{
        let index = self.config.connections.iter().position(|c| c.uuid == connection_uuid);
        let index = if let Some(index) = index {
            index
        } else {
            return Err(format!("Connection '{}' not found", connection_uuid));
        };
        let _ = SecureStorage::remove_db_password(&connection_uuid.to_string());
        self.config.connections.remove(index);
        self.config.save();
        Ok(())
    }
    pub fn save_settings(&mut self) -> Result<(), String> {
        self.config.llm_api.provider = self.settings.provider.clone();
        self.config.llm_api.model = self.settings.model.clone();
        // Save API key securely
        if !self.settings.api_key.is_empty() {
            if let Err(err) = SecureStorage::store_api_key(
                &self.settings.api_key
            ) {
                return Err(err.to_string());
            }
            self.config.save();
        }
        Ok(())
    }

    // pub fn send_message(&mut self) {
    //     if self.current_message.trim().is_empty() {
    //         return;
    //     }
    //
    //     // Add user message to db_element
    //     let message = self.current_message.clone();
    //     self.chat_messages.push(ChatMessage {
    //         sender: MessageSender::User,
    //         content: message.clone(),
    //         is_sql: false,
    //     });
    //     self.current_message.clear();
    //
    //     // Check if we have an active connection
    //     if self.active_connection.is_none() {
    //         self.chat_messages.push(ChatMessage {
    //             sender: MessageSender::System,
    //             content: "Please select an active database connection first.".to_string(),
    //             is_sql: false,
    //         });
    //         return;
    //     }
    //
    //     let active_connection = self.active_connection.clone().unwrap();
    //     let db_manager = self.db_manager.clone();
    //     let llm_client = match &self.llm_client {
    //         Some(client) => (*client).clone(),
    //         None => {
    //             self.chat_messages.push(ChatMessage {
    //                 sender: MessageSender::System,
    //                 content: "LLM API is not configured. Please set up API settings first.".to_string(),
    //                 is_sql: false,
    //             });
    //             return;
    //         }
    //     };
    //
    //     // Get schema info and generate SQL (asynchronously)
    //     let message_clone = message.clone();
    //
    //     // Use an oneshot channel to get the result back to the UI thread
    //     let (tx, _rx) = tokio::sync::oneshot::channel();
    //
    //
    //     self.runtime.spawn(async move {
    //         // Get schema info
    //         // let schema_info = match db_manager.get_schema_info(&active_connection).await {
    //         //     Ok(info) => info,
    //         //     Err(err) => {
    //         //         let _ = tx.send(Err(format!("Failed to get schema info: {}", err)));
    //         //         return;
    //         //     }
    //         // };
    //         //
    //         // // Generate SQL query
    //         // let sql = match llm_client.generate_sql(&message_clone, &schema_info).await {
    //         //     Ok(sql) => sql,
    //         //     Err(err) => {
    //         //         let _ = tx.send(Err(format!("Failed to generate SQL: {}", err)));
    //         //         return;
    //         //     }
    //         // };
    //         //
    //         // // Execute the query
    //         // match db_manager.execute_query(&active_connection, &sql).await {
    //         //     Ok(result) => {
    //         //         let _ = tx.send(Ok((sql, result)));
    //         //     },
    //         //     Err(err) => {
    //         //         let _ = tx.send(Err(format!("Failed to execute query: {}", err)));
    //         //     }
    //         // }
    //
    //     });
    //
    //     // Store the receiver for later use
    //     // In a real application, you would use a state machine or callback mechanism
    //     // For simplicity, we'll assume the UI will check for the result on the next frame
    //     // This is a placeholder and not fully implemented
    //     // self.pending_query = Some(rx);
    // }
}

impl eframe::App for DBQueryApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {

        let mut visuals = egui::Visuals::default();
        visuals.panel_fill = egui::Color32::BLACK; // Black background
        visuals.override_text_color = Some(egui::Color32::from_rgb(245, 245, 245)); // Global font color (yellow-orange)
        ctx.set_visuals(visuals);

        // Render the UI
        render_ui(ctx, &mut self.state);

        // Request a repaint for continuous UI updates
        ctx.request_repaint();
    }
}