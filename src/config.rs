use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use crate::error::{CCSwitchError, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Channel {
    pub name: String,
    pub url: String,
    pub api_key: Option<String>,
    pub model: Option<String>,
    pub enabled: bool,
    pub priority: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub channels: HashMap<String, Channel>,
    pub default_model: Option<String>,
    pub timeout_seconds: u64,
    pub retry_attempts: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            channels: HashMap::new(),
            default_model: None,
            timeout_seconds: 30,
            retry_attempts: 3,
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;
        
        if !config_path.exists() {
            // Create default config if it doesn't exist
            let config = Config::default();
            config.save()?;
            return Ok(config);
        }
        
        let content = fs::read_to_string(&config_path)
            .map_err(|e| CCSwitchError::Config(format!("Failed to read config file: {}", e)))?;
            
        serde_json::from_str(&content)
            .map_err(|e| CCSwitchError::Config(format!("Failed to parse config file: {}", e)))
    }
    
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;
        
        // Create config directory if it doesn't exist
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| CCSwitchError::Config(format!("Failed to create config directory: {}", e)))?;
        }
        
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| CCSwitchError::Config(format!("Failed to serialize config: {}", e)))?;
            
        fs::write(&config_path, content)
            .map_err(|e| CCSwitchError::Config(format!("Failed to write config file: {}", e)))?;
            
        Ok(())
    }
    
    pub fn add_channel(&mut self, channel: Channel) -> Result<()> {
        if self.channels.contains_key(&channel.name) {
            return Err(CCSwitchError::Config(format!("Channel '{}' already exists", channel.name)));
        }
        
        self.channels.insert(channel.name.clone(), channel);
        self.save()
    }
    
    pub fn remove_channel(&mut self, name: &str) -> Result<()> {
        if !self.channels.contains_key(name) {
            return Err(CCSwitchError::ChannelNotFound(name.to_string()));
        }
        
        self.channels.remove(name);
        self.save()
    }
    
    pub fn get_channel(&self, name: &str) -> Option<&Channel> {
        self.channels.get(name)
    }
    
    pub fn get_channels_for_model(&self, model: &str) -> Vec<&Channel> {
        self.channels
            .values()
            .filter(|ch| ch.enabled && (ch.model.as_deref() == Some(model) || ch.model.is_none()))
            .collect()
    }
    
    fn config_path() -> Result<PathBuf> {
        dirs::config_dir()
            .map(|mut path| {
                path.push("ccswitch");
                path.push("config.json");
                path
            })
            .ok_or_else(|| CCSwitchError::Config("Could not determine config directory".to_string()))
    }
}