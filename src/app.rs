use crate::config::{AppConfig, DbConnection};
use crate::db::{DatabaseManager, QueryResult};
use crate::llm::llm::LLMClient;
use crate::security::{SecureStorage, SecurityError};
use crate::ui::setting::Settings;
use crate::ui::ui::render_ui;
use eframe::egui;
use std::sync::Arc;
use tokio::runtime::Runtime;

pub enum AppMode {
    Query,
    Connections,
    Settings,
    NewConnection,
}

pub struct AppState {
    pub config: AppConfig,
    pub settings: Settings,
    pub mode: AppMode,
    pub db_manager: Arc<DatabaseManager>,
    pub llm_client: Option<LLMClient>,
    pub active_connection: Option<String>,
    pub chat_messages: Vec<ChatMessage>,
    pub current_message: String,
    pub query_result: Option<QueryResult>,
    pub new_connection: Option<DbConnection>,
    pub new_connection_password: Option<String>,
    pub editing_connection: bool,
    pub error_message: Option<String>,
    pub success_message: Option<String>,
    pub runtime: Runtime,
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
                },
                mode: AppMode::Query,
                db_manager,
                llm_client,
                active_connection: None,
                chat_messages: Vec::new(),
                current_message: String::new(),
                query_result: None,
                new_connection: None,
                new_connection_password: None,
                editing_connection: false,
                error_message: None,
                success_message: None,
                runtime,
            },
        }
    }
}

impl AppState {
    pub fn test_connection(&mut self, connection_name: String) {
        let db_manager_clone = self.db_manager.clone();
        let connections = self.config.connections.clone();

        // Find the connection config
        let connection = match connections.iter().find(|c| c.name == connection_name) {
            Some(conn) => conn.clone(),
            None => {
                self.error_message = Some(format!("Connection '{}' not found", connection_name));
                return;
            }
        };

        // Test the connection asynchronously
        self.runtime.spawn(async move {
            if let Err(err) = db_manager_clone.connect(&connection).await {
                // Handle error in UI thread
                // This would require a channel or another mechanism to communicate back to the UI thread
                // For simplicity, we'll just print the error
                eprintln!("Connection test failed: {}", err);
            } else {
                eprintln!("Connection test successful");
            }
        });
    }

    pub fn save_connection(&mut self) {
        if let Some(connection) = self.new_connection.take() {
            if let Some(password) = self.new_connection_password.take() {
                // Store password securely
                if let Err(err) = SecureStorage::store_db_password(&connection.name, &password) {
                    self.error_message = Some(format!("Failed to store password: {}", err));
                    return;
                }

                // Update or add the connection
                if self.editing_connection {
                    if let Some(idx) = self.config.connections.iter().position(|c| c.name == connection.name) {
                        self.config.connections[idx] = connection;
                    } else {
                        self.config.connections.push(connection);
                    }
                } else {
                    self.config.connections.push(connection);
                }

                // Save the config
                self.config.save();
                self.success_message = Some("Connection saved successfully!".to_string());
                self.mode = AppMode::Connections;
                self.editing_connection = false;
            }
        }
    }

    pub fn send_message(&mut self) {
        if self.current_message.trim().is_empty() {
            return;
        }

        // Add user message to chat
        let message = self.current_message.clone();
        self.chat_messages.push(ChatMessage {
            sender: MessageSender::User,
            content: message.clone(),
            is_sql: false,
        });
        self.current_message.clear();

        // Check if we have an active connection
        if self.active_connection.is_none() {
            self.chat_messages.push(ChatMessage {
                sender: MessageSender::System,
                content: "Please select an active database connection first.".to_string(),
                is_sql: false,
            });
            return;
        }

        let active_connection = self.active_connection.clone().unwrap();
        let db_manager = self.db_manager.clone();
        let llm_client = match &self.llm_client {
            Some(client) => (*client).clone(),
            None => {
                self.chat_messages.push(ChatMessage {
                    sender: MessageSender::System,
                    content: "LLM API is not configured. Please set up API settings first.".to_string(),
                    is_sql: false,
                });
                return;
            }
        };

        // Get schema info and generate SQL (asynchronously)
        let message_clone = message.clone();

        // Use a oneshot channel to get the result back to the UI thread
        let (tx, _rx) = tokio::sync::oneshot::channel();


        self.runtime.spawn(async move {
            // Get schema info
            let schema_info = match db_manager.get_schema_info(&active_connection).await {
                Ok(info) => info,
                Err(err) => {
                    let _ = tx.send(Err(format!("Failed to get schema info: {}", err)));
                    return;
                }
            };

            // Generate SQL query
            let sql = match llm_client.generate_sql(&message_clone, &schema_info).await {
                Ok(sql) => sql,
                Err(err) => {
                    let _ = tx.send(Err(format!("Failed to generate SQL: {}", err)));
                    return;
                }
            };

            // Execute the query
            match db_manager.execute_query(&active_connection, &sql).await {
                Ok(result) => {
                    let _ = tx.send(Ok((sql, result)));
                },
                Err(err) => {
                    let _ = tx.send(Err(format!("Failed to execute query: {}", err)));
                }
            }

        });

        // Store the receiver for later use
        // In a real application, you would use a state machine or callback mechanism
        // For simplicity, we'll assume the UI will check for the result on the next frame
        // This is a placeholder and not fully implemented
        // self.pending_query = Some(rx);
    }
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