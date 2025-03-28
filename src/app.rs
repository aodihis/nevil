use crate::config::{get_chat_db_path, AppConfig, DbConnection};
use crate::db_element::chat_storage::ChatStorage;
use crate::db_element::db::DatabaseManager;
use crate::llm::llm::LLMClient;
use crate::security::SecureStorage;
use crate::ui::chat::Conversation;
use crate::ui::connection::Connection;
use crate::ui::query_result::ResultTable;
use crate::ui::setting::Settings;
use crate::ui::ui::render_ui;
use eframe::egui;
use std::sync::Arc;
use tokio::runtime::Runtime;
use uuid::Uuid;

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
    pub chat_storage: Arc<ChatStorage>,
    pub llm_client: Option<LLMClient>,

    pub runtime: Runtime,
    pub query_tx: tokio::sync::mpsc::Sender<Result<ResultTable, String>>,
    pub query_rx: tokio::sync::mpsc::Receiver<Result<ResultTable, String>>,

    // UI related struct
    pub settings: Settings,
    pub connection: Connection,
    pub conversation: Conversation,
    pub query_result: Vec<ResultTable>,
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
        let chat_path = get_chat_db_path();
        let (tx, rx) = tokio::sync::mpsc::channel(1);
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
                chat_storage: Arc::new(ChatStorage::new(chat_path).unwrap()),
                llm_client,
                query_result: Vec::new(),
                connection: Connection::new(),
                runtime,
                query_tx: tx,
                query_rx: rx,
                conversation: Conversation::new(None),
            },
        }
    }
}

impl AppState {
    pub fn save_db(&mut self) -> Result<(), String> {
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

    pub fn remove_db(&mut self, uuid: Uuid) -> Result<(), String>{
        let index = self.config.connections.iter().position(|c| c.uuid == uuid);
        let index = if let Some(index) = index {
            index
        } else {
            return Err(format!("Connection '{}' not found", uuid));
        };
        let _ = SecureStorage::remove_db_password(&uuid.to_string());
        let _ = self.chat_storage.remove_conversation(&uuid);
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

    pub fn run_query(&self, connection_id: &Uuid, query: &str, message_uuid: &Uuid) {

        let tx = self.query_tx.clone();
        let message_uuid = message_uuid.clone();
        let db_manager = self.db_manager.clone();
        let connection_id = connection_id.clone();
        let query = query.to_string();
        self.runtime.spawn(async move {
            let res = match db_manager.execute_query(&connection_id, &query).await {
                Ok(res) => {
                    res
                },
                Err(e) => {
                    tx.send(Err(format!("Failed to execute query: {}", e))).await.ok();
                    return;
                },
            };
            let table = ResultTable {
                id: message_uuid,
                title: query,
                data: res,
                current_page: 0,
                total_of_pages: 0,
                is_open: true,
            };
            tx.send(Ok(table)).await.ok();
        });

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