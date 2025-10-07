use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConnection {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub database: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedQuery {
    pub name: String,
    pub sql: String,
    #[serde(default)]
    pub created_at: String,
}

impl DatabaseConnection {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            host: "localhost".to_string(),
            port: 5432,
            user: "postgres".to_string(),
            password: String::new(),
            database: "postgres".to_string(),
        }
    }

    pub fn to_connection_string(&self) -> String {
        format!(
            "host={} port={} user={} password={} dbname={}",
            self.host, self.port, self.user, self.password, self.database
        )
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub connections: Vec<DatabaseConnection>,
    pub last_connection_index: Option<usize>,
}

impl Config {
    pub fn new() -> Self {
        Self {
            connections: vec![],
            last_connection_index: None,
        }
    }

    pub fn load() -> Result<Self> {
        let config_path = Self::get_config_path()?;

        if config_path.exists() {
            let content = fs::read_to_string(&config_path)?;
            let config: Config = serde_json::from_str(&content)?;
            Ok(config)
        } else {
            Ok(Self::new())
        }
    }

    pub fn save(&self) -> Result<()> {
        let config_path = Self::get_config_path()?;

        // Create parent directory if it doesn't exist
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(self)?;
        fs::write(&config_path, content)?;
        Ok(())
    }

    fn get_config_path() -> Result<PathBuf> {
        let home = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
        Ok(home.join(".config").join("db-client").join("config.json"))
    }

    pub fn add_connection(&mut self, connection: DatabaseConnection) {
        self.connections.push(connection);
    }

    pub fn update_connection(&mut self, index: usize, connection: DatabaseConnection) {
        if index < self.connections.len() {
            self.connections[index] = connection;
        }
    }

    pub fn delete_connection(&mut self, index: usize) {
        if index < self.connections.len() {
            self.connections.remove(index);

            // Update last_connection_index if needed
            if let Some(last_idx) = self.last_connection_index {
                if last_idx == index {
                    self.last_connection_index = None;
                } else if last_idx > index {
                    self.last_connection_index = Some(last_idx - 1);
                }
            }
        }
    }

    pub fn get_connection(&self, index: usize) -> Option<&DatabaseConnection> {
        self.connections.get(index)
    }

    pub fn get_last_connection(&self) -> Option<&DatabaseConnection> {
        self.last_connection_index
            .and_then(|idx| self.connections.get(idx))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SavedQueries {
    pub queries: Vec<SavedQuery>,
}

impl SavedQueries {
    pub fn new() -> Self {
        Self {
            queries: vec![],
        }
    }

    pub fn load() -> Result<Self> {
        let queries_path = Self::get_queries_path()?;

        if queries_path.exists() {
            let content = fs::read_to_string(&queries_path)?;
            let queries: SavedQueries = serde_json::from_str(&content)?;
            Ok(queries)
        } else {
            Ok(Self::new())
        }
    }

    pub fn save(&self) -> Result<()> {
        let queries_path = Self::get_queries_path()?;

        // Create parent directory if it doesn't exist
        if let Some(parent) = queries_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(self)?;
        fs::write(&queries_path, content)?;
        Ok(())
    }

    fn get_queries_path() -> Result<PathBuf> {
        let home = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
        Ok(home.join(".config").join("db-client").join("queries.json"))
    }

    pub fn add_query(&mut self, name: String, sql: String) {
        let created_at = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        self.queries.push(SavedQuery {
            name,
            sql,
            created_at,
        });
    }

    pub fn delete_query(&mut self, index: usize) {
        if index < self.queries.len() {
            self.queries.remove(index);
        }
    }

    pub fn get_query(&self, index: usize) -> Option<&SavedQuery> {
        self.queries.get(index)
    }
}
