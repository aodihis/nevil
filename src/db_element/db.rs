use std::collections::HashMap;
use crate::config::{DbConnection, DbType};
use crate::security::SecureStorage;
use sqlx::{mysql::MySqlPoolOptions, postgres::PgPoolOptions, MySqlPool, PgPool, Row};
use std::sync::Arc;
use std::time::Duration;
use log::{debug};
use tokio::sync::Mutex;
use uuid::Uuid;
use crate::utils::db_utils::{mysql_query, postgres_query};

pub const PAGE_SIZE: usize = 100;
pub enum DbPool {
    MySQL(MySqlPool),
    PostgreSQL(PgPool),
}

pub struct DatabaseManager {
    connections: Arc<Mutex<HashMap<Uuid, DbPool>>>,
}

#[derive(Debug)]
pub struct QueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub current_page: usize,
    pub total_pages: usize,
    #[allow(dead_code)]
    pub limit: usize,
}

impl DatabaseManager {
    pub fn new() -> Self {
        Self {
            connections: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn connect(&self, connection: &DbConnection, password: Option<String>, is_temp: bool) -> Result<(), String> {
        // Get password securely
        let password = if password.is_some() {
            password.unwrap()
        } else {
            match SecureStorage::get_db_password(&connection.name) {
                Ok(pwd) => pwd,
                Err(_) => return Err("Password not found".to_string()),
            }
        };

        // Build connection string
        let connection_string = connection.connection_string_template()
            .replace("{host}", &connection.host)
            .replace("{port}", &connection.port.to_string())
            .replace("{username}", &connection.username)
            .replace("{password}", &password)
            .replace("{database}", &connection.database);

        let timeout_duration = Duration::from_secs(5);
        // Create pool based on database type
        let pool = match connection.db_type {
            DbType::MySQL => {
                let pool = MySqlPoolOptions::new()
                    .acquire_timeout(timeout_duration)
                    .max_connections(5)
                    .connect(&connection_string)
                    .await
                    .map_err(|e| e.to_string())?;
                DbPool::MySQL(pool)
            },
            DbType::PostgreSQL => {
                let pool = PgPoolOptions::new()
                    .acquire_timeout(timeout_duration)
                    .max_connections(5)
                    .connect(&connection_string)
                    .await
                    .map_err(|e| e.to_string())?;
                DbPool::PostgreSQL(pool)
            },
        };

        if !is_temp {
            // Store the connection
            let mut connections = self.connections.lock().await;
            connections.insert(connection.uuid.clone(), pool);
        }

        Ok(())
    }

    pub async fn get_schema_info(&self, connection_uuid: &Uuid) -> Result<String, String> {
        let connections = self.connections.lock().await;

        let connection = connections.get(connection_uuid).ok_or_else(|| "Connection not found".to_string())?;
        // Query schema information based on database type
        match connection {
            DbPool::MySQL(pool) => {
                // For MySQL, query information_schema for table and column information
                let tables = sqlx::query(
                    "SELECT table_name, table_comment FROM information_schema.tables
                     WHERE table_schema = DATABASE() ORDER BY table_name"
                )
                    .fetch_all(pool)
                    .await
                    .map_err(|e| e.to_string())?;

                let mut schema = String::new();

                for table in tables {
                    let table_name: String = table.get("table_name");
                    schema.push_str(&format!("Table: {}\n", table_name));

                    let columns = sqlx::query(
                        "SELECT column_name, data_type, column_comment,
                                is_nullable, column_key
                         FROM information_schema.columns
                         WHERE table_schema = DATABASE() AND table_name = ?
                         ORDER BY ordinal_position"
                    )
                        .bind(&table_name)
                        .fetch_all(pool)
                        .await
                        .map_err(|e| e.to_string())?;

                    for column in columns {
                        let column_name: String = column.get("column_name");
                        let data_type: String = column.get("data_type");
                        let is_nullable: String = column.get("is_nullable");
                        let column_key: Option<String> = column.try_get("column_key").ok();

                        schema.push_str(&format!(
                            "  - {} ({}, {}{})\n",
                            column_name,
                            data_type,
                            if is_nullable == "YES" { "NULL" } else { "NOT NULL" },
                            if let Some(key) = column_key {
                                if key == "PRI" { ", PRIMARY KEY" } else if key == "UNI" { ", UNIQUE" } else { "" }
                            } else { "" }
                        ));
                    }

                    schema.push('\n');
                }

                Ok(schema)
            },
            DbPool::PostgreSQL(pool) => {
                // For PostgreSQL, query information schema for table and column information
                let tables = sqlx::query(
                    "SELECT table_name, obj_description(pgc.oid) as table_comment
                     FROM pg_catalog.pg_class pgc
                     JOIN information_schema.tables t ON pgc.relname = t.table_name
                     WHERE t.table_schema = 'public' AND t.table_type = 'BASE TABLE'
                     ORDER BY table_name"
                )
                    .fetch_all(pool)
                    .await
                    .map_err(|e| e.to_string())?;

                let mut schema = String::new();

                for table in tables {
                    let table_name: String = table.get("table_name");
                    schema.push_str(&format!("Table: {}\n", table_name));

                    let columns = sqlx::query(
                        "SELECT c.column_name, c.data_type, c.is_nullable,
                                pg_catalog.col_description(format('%s.%s', c.table_schema, c.table_name)::regclass::oid, c.ordinal_position) as column_comment,
                                tc.constraint_type
                         FROM information_schema.columns c
                         LEFT JOIN information_schema.constraint_column_usage ccu ON c.column_name = ccu.column_name AND c.table_name = ccu.table_name
                         LEFT JOIN information_schema.table_constraints tc ON ccu.constraint_name = tc.constraint_name
                         WHERE c.table_schema = 'public' AND c.table_name = $1
                         ORDER BY c.ordinal_position"
                    )
                        .bind(&table_name)
                        .fetch_all(pool)
                        .await
                        .map_err(|e| e.to_string())?;

                    for column in columns {
                        let column_name: String = column.get("column_name");
                        let data_type: String = column.get("data_type");
                        let is_nullable: String = column.get("is_nullable");
                        let constraint_type: Option<String> = column.try_get("constraint_type").ok();

                        schema.push_str(&format!(
                            "  - {} ({}, {}{})\n",
                            column_name,
                            data_type,
                            if is_nullable == "YES" { "NULL" } else { "NOT NULL" },
                            if let Some(ctype) = constraint_type {
                                if ctype == "PRIMARY KEY" { ", PRIMARY KEY" } else if ctype == "UNIQUE" { ", UNIQUE" } else { "" }
                            } else { "" }
                        ));
                    }

                    schema.push('\n');
                }

                Ok(schema)
            }
        }
    }

    pub async fn execute_query(&self, connection_uuid: &Uuid, query: &str, offset: usize, limit: Option<usize>) -> Result<QueryResult, String> {
        debug!("Start running query: {}", query);
        let connections = self.connections.lock().await;
        // Find the connection
        let connection = connections.get(connection_uuid).ok_or_else(|| "Connection not found".to_string())?;
        let limit = limit.unwrap_or(PAGE_SIZE);

        let query = query.trim_end_matches(';');
        let count_query = format!("SELECT COUNT(*) FROM ({}) AS subquery", query);
        let paginated_query = format!("SELECT * FROM ({}) AS subquery LIMIT {} OFFSET {}", query, limit, offset);

        debug!("Count query: {}", count_query);
        debug!("Paginated query: {}", count_query);
        // Execute query based on database type
        match connection {
            DbPool::MySQL(pool) => {
                mysql_query(pool, count_query, paginated_query, offset, limit).await

            },
            DbPool::PostgreSQL(pool) => {
                postgres_query(pool, count_query, paginated_query, offset, limit).await
            }
        }
    }
}

