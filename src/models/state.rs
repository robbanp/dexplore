use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use anyhow::Result;
use crate::models::Tab;

#[derive(Serialize, Deserialize)]
pub struct AppState {
    pub tabs: Vec<Tab>,
    pub active_tab: usize,
    pub next_tab_id: usize,
    pub expanded_schemas: HashSet<String>,
}

impl AppState {
    pub fn save_path() -> Result<PathBuf> {
        let home = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
        Ok(home.join(".config").join("db-client").join("state.json"))
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::save_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        fs::write(&path, content)?;
        Ok(())
    }

    pub fn load() -> Result<Self> {
        let path = Self::save_path()?;
        if path.exists() {
            let content = fs::read_to_string(&path)?;
            let state: AppState = serde_json::from_str(&content)?;
            Ok(state)
        } else {
            Err(anyhow::anyhow!("State file does not exist"))
        }
    }
}
