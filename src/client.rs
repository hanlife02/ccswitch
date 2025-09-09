use crate::config::Channel;
use crate::channel::ChannelManager;
use crate::error::{CCSwitchError, Result};
use reqwest::Client;
use serde_json::{json, Value};
use std::time::Duration;
use log::{info, error};

pub struct APIClient {
    channel_manager: ChannelManager,
    client: Client,
}

#[derive(Debug)]
pub struct RequestOptions {
    pub model: Option<String>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub stream: bool,
}

impl Default for RequestOptions {
    fn default() -> Self {
        Self {
            model: None,
            max_tokens: Some(1000),
            temperature: Some(0.7),
            stream: false,
        }
    }
}

#[derive(Debug)]
pub struct APIResponse {
    pub content: String,
    pub channel_used: String,
    pub model: String,
    pub usage: Option<Value>,
}

impl APIClient {
    pub fn new() -> Result<Self> {
        let channel_manager = ChannelManager::new()?;
        let client = Client::builder()
            .timeout(Duration::from_secs(60))
            .build()
            .map_err(CCSwitchError::Network)?;
            
        Ok(Self {
            channel_manager,
            client,
        })
    }
    
    pub async fn make_request(&mut self, prompt: &str, options: RequestOptions) -> Result<APIResponse> {
        let model = options.model
            .as_deref()
            .or(self.channel_manager.config.default_model.as_deref())
            .unwrap_or("gpt-3.5-turbo");
            
        info!("Making request for model: {}", model);
        
        // Find an available channel for the model
        let channel = self.channel_manager.find_available_channel(model).await?;
        
        // Prepare the request payload
        let payload = json!({
            "model": model,
            "messages": [
                {
                    "role": "user",
                    "content": prompt
                }
            ],
            "max_tokens": options.max_tokens,
            "temperature": options.temperature,
            "stream": options.stream
        });
        
        // Make the request
        let response = self.send_request(channel, &payload).await?;
        
        // Parse the response
        self.parse_response(response, channel.name.clone(), model.to_string()).await
    }
    
    async fn send_request(&self, channel: &Channel, payload: &Value) -> Result<reqwest::Response> {
        info!("Sending request to channel: {}", channel.name);
        
        let mut request = self.client.post(&channel.url);
        
        // Add authentication if available
        if let Some(api_key) = &channel.api_key {
            request = request.header("Authorization", format!("Bearer {}", api_key));
        }
        
        // Send the request
        request = request
            .header("Content-Type", "application/json")
            .json(payload);
            
        let response = request.send().await
            .map_err(|e| {
                error!("Request failed for channel {}: {}", channel.name, e);
                CCSwitchError::Network(e)
            })?;
            
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            error!("API request failed with status {}: {}", status, error_text);
            return Err(CCSwitchError::Channel(format!("API request failed: {} - {}", status, error_text)));
        }
        
        Ok(response)
    }
    
    async fn parse_response(&self, response: reqwest::Response, channel_name: String, model: String) -> Result<APIResponse> {
        let response_text = response.text().await
            .map_err(CCSwitchError::Network)?;
            
        let json_response: Value = serde_json::from_str(&response_text)
            .map_err(|e| CCSwitchError::Channel(format!("Failed to parse response: {}", e)))?;
            
        // Extract content from different response formats
        let content = self.extract_content(&json_response)?;
        let usage = json_response.get("usage").cloned();
        
        Ok(APIResponse {
            content,
            channel_used: channel_name,
            model,
            usage,
        })
    }
    
    fn extract_content(&self, response: &Value) -> Result<String> {
        // Try OpenAI format first
        if let Some(choices) = response.get("choices").and_then(|c| c.as_array()) {
            if let Some(first_choice) = choices.first() {
                if let Some(message) = first_choice.get("message") {
                    if let Some(content) = message.get("content").and_then(|c| c.as_str()) {
                        return Ok(content.to_string());
                    }
                }
                
                // Try delta format for streaming
                if let Some(delta) = first_choice.get("delta") {
                    if let Some(content) = delta.get("content").and_then(|c| c.as_str()) {
                        return Ok(content.to_string());
                    }
                }
            }
        }
        
        // Try Claude format
        if let Some(content) = response.get("content") {
            if let Some(text) = content.as_str() {
                return Ok(text.to_string());
            }
            
            if let Some(content_array) = content.as_array() {
                if let Some(first_content) = content_array.first() {
                    if let Some(text) = first_content.get("text").and_then(|t| t.as_str()) {
                        return Ok(text.to_string());
                    }
                }
            }
        }
        
        // Fallback: try to extract any string field that might contain the response
        if let Some(text) = response.get("text").and_then(|t| t.as_str()) {
            return Ok(text.to_string());
        }
        
        if let Some(response_text) = response.get("response").and_then(|t| t.as_str()) {
            return Ok(response_text.to_string());
        }
        
        Err(CCSwitchError::Channel("Could not extract content from response".to_string()))
    }
    
    pub fn reload_config(&mut self) -> Result<()> {
        self.channel_manager.reload_config()
    }
    
    pub fn get_channel_manager(&self) -> &ChannelManager {
        &self.channel_manager
    }
    
    pub fn get_channel_manager_mut(&mut self) -> &mut ChannelManager {
        &mut self.channel_manager
    }
}