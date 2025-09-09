use crate::config::{Channel, Config};
use crate::error::{CCSwitchError, Result};
use reqwest::Client;
use serde_json::json;
use std::time::Duration;
use log::{debug, warn, error};

pub struct ChannelManager {
    pub config: Config,
    client: Client,
}

#[derive(Debug)]
pub struct ChannelStatus {
    pub name: String,
    pub available: bool,
    pub response_time_ms: Option<u64>,
    pub error: Option<String>,
}

impl ChannelManager {
    pub fn new() -> Result<Self> {
        let config = Config::load()?;
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .build()
            .map_err(CCSwitchError::Network)?;
            
        Ok(Self { config, client })
    }
    
    pub fn reload_config(&mut self) -> Result<()> {
        self.config = Config::load()?;
        Ok(())
    }
    
    pub fn add_channel(&mut self, name: String, url: String, api_key: Option<String>, model: Option<String>) -> Result<()> {
        let channel = Channel {
            name: name.clone(),
            url,
            api_key,
            model,
            enabled: true,
            priority: 0,
        };
        
        self.config.add_channel(channel)?;
        Ok(())
    }
    
    pub fn remove_channel(&mut self, name: &str) -> Result<()> {
        self.config.remove_channel(name)?;
        Ok(())
    }
    
    pub fn list_channels(&self) -> Vec<&Channel> {
        self.config.channels.values().collect()
    }
    
    pub async fn test_channel(&self, channel: &Channel) -> ChannelStatus {
        debug!("Testing channel: {}", channel.name);
        
        let start = std::time::Instant::now();
        
        // Create a simple test request
        let test_payload = json!({
            "model": channel.model.as_deref().unwrap_or("test"),
            "messages": [
                {
                    "role": "user",
                    "content": "Hello"
                }
            ],
            "max_tokens": 1
        });
        
        let mut request = self.client.post(&channel.url);
        
        if let Some(api_key) = &channel.api_key {
            request = request.header("Authorization", format!("Bearer {}", api_key));
        }
        
        request = request
            .header("Content-Type", "application/json")
            .json(&test_payload);
        
        match request.send().await {
            Ok(response) => {
                let response_time = start.elapsed().as_millis() as u64;
                let status_code = response.status();
                
                if status_code.is_success() || status_code.as_u16() == 400 {
                    // 400 might be OK for test requests with invalid model
                    debug!("Channel {} is available (response time: {}ms)", channel.name, response_time);
                    ChannelStatus {
                        name: channel.name.clone(),
                        available: true,
                        response_time_ms: Some(response_time),
                        error: None,
                    }
                } else {
                    let error = format!("HTTP {}: {}", status_code, status_code.canonical_reason().unwrap_or("Unknown"));
                    warn!("Channel {} returned error: {}", channel.name, error);
                    ChannelStatus {
                        name: channel.name.clone(),
                        available: false,
                        response_time_ms: Some(response_time),
                        error: Some(error),
                    }
                }
            }
            Err(e) => {
                error!("Channel {} failed: {}", channel.name, e);
                ChannelStatus {
                    name: channel.name.clone(),
                    available: false,
                    response_time_ms: None,
                    error: Some(e.to_string()),
                }
            }
        }
    }
    
    pub async fn test_all_channels(&self) -> Vec<ChannelStatus> {
        let mut results = Vec::new();
        
        for channel in self.config.channels.values() {
            if channel.enabled {
                let status = self.test_channel(channel).await;
                results.push(status);
            }
        }
        
        results
    }
    
    pub async fn find_available_channel(&self, model: &str) -> Result<&Channel> {
        let channels = self.config.get_channels_for_model(model);
        
        if channels.is_empty() {
            return Err(CCSwitchError::NoAvailableChannels(model.to_string()));
        }
        
        // Test channels in priority order
        let mut sorted_channels = channels;
        sorted_channels.sort_by_key(|ch| ch.priority);
        
        for channel in sorted_channels {
            let status = self.test_channel(channel).await;
            if status.available {
                return Ok(channel);
            }
        }
        
        Err(CCSwitchError::AllChannelsFailed)
    }
}