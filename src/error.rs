use thiserror::Error;

#[derive(Error, Debug)]
pub enum CCSwitchError {
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("Channel error: {0}")]
    Channel(String),
    
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Channel '{0}' not found")]
    ChannelNotFound(String),
    
    #[error("No available channels for model '{0}'")]
    NoAvailableChannels(String),
    
    #[error("All channels failed")]
    AllChannelsFailed,
}

pub type Result<T> = std::result::Result<T, CCSwitchError>;