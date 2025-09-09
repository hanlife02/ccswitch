mod config;
mod channel;
mod client;
mod error;

use clap::{Parser, Subcommand};
use channel::ChannelManager;
use client::{APIClient, RequestOptions};
use error::Result;
use log::info;

#[derive(Parser)]
#[command(name = "ccswitch")]
#[command(about = "A CLI tool for automatic switching between multiple model API channels")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a new channel configuration
    Add {
        /// Channel name
        name: String,
        /// API endpoint URL
        url: String,
        /// API key
        #[arg(short, long)]
        key: Option<String>,
        /// Model name
        #[arg(short, long)]
        model: Option<String>,
    },
    /// List all configured channels
    List,
    /// Remove a channel
    Remove {
        /// Channel name to remove
        name: String,
    },
    /// Test channel availability
    Test {
        /// Channel name to test (if not specified, test all)
        name: Option<String>,
    },
    /// Make a request with automatic channel switching
    Request {
        /// The prompt/message to send
        prompt: String,
        /// Preferred model name
        #[arg(short, long)]
        model: Option<String>,
        /// Maximum tokens
        #[arg(long)]
        max_tokens: Option<u32>,
        /// Temperature (0.0-2.0)
        #[arg(short, long)]
        temperature: Option<f32>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Add { name, url, key, model } => {
            info!("Adding channel: {}", name);
            let mut manager = ChannelManager::new()?;
            manager.add_channel(name.clone(), url, key, model)?;
            println!("✓ Channel '{}' added successfully", name);
        }
        Commands::List => {
            info!("Listing all channels");
            let manager = ChannelManager::new()?;
            let channels = manager.list_channels();
            
            if channels.is_empty() {
                println!("No channels configured");
            } else {
                println!("Configured channels:");
                for channel in channels {
                    let status = if channel.enabled { "enabled" } else { "disabled" };
                    let model_info = channel.model.as_deref().unwrap_or("any");
                    println!("  {} [{}] - {} (model: {})", 
                        channel.name, status, channel.url, model_info);
                }
            }
        }
        Commands::Remove { name } => {
            info!("Removing channel: {}", name);
            let mut manager = ChannelManager::new()?;
            manager.remove_channel(&name)?;
            println!("✓ Channel '{}' removed successfully", name);
        }
        Commands::Test { name } => {
            info!("Testing channel availability");
            let manager = ChannelManager::new()?;
            
            match name {
                Some(channel_name) => {
                    if let Some(channel) = manager.config.get_channel(&channel_name) {
                        println!("Testing channel: {}", channel_name);
                        let status = manager.test_channel(channel).await;
                        print_channel_status(&status);
                    } else {
                        println!("❌ Channel '{}' not found", channel_name);
                    }
                }
                None => {
                    println!("Testing all channels:");
                    let results = manager.test_all_channels().await;
                    for status in results {
                        print_channel_status(&status);
                    }
                }
            }
        }
        Commands::Request { prompt, model, max_tokens, temperature } => {
            info!("Making request with prompt: {}", prompt);
            
            let mut client = APIClient::new()?;
            let options = RequestOptions {
                model,
                max_tokens,
                temperature,
                stream: false,
            };
            
            match client.make_request(&prompt, options).await {
                Ok(response) => {
                    println!("✓ Response from {} (model: {}):", response.channel_used, response.model);
                    println!("{}", response.content);
                    
                    if let Some(usage) = response.usage {
                        println!("\nUsage: {}", usage);
                    }
                }
                Err(e) => {
                    eprintln!("❌ Request failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
    }
    
    Ok(())
}

fn print_channel_status(status: &channel::ChannelStatus) {
    let icon = if status.available { "✓" } else { "❌" };
    let mut message = format!("{} {} - {}", 
        icon, 
        status.name, 
        if status.available { "Available" } else { "Unavailable" }
    );
    
    if let Some(response_time) = status.response_time_ms {
        message.push_str(&format!(" ({}ms)", response_time));
    }
    
    if let Some(error) = &status.error {
        message.push_str(&format!(" - {}", error));
    }
    
    println!("  {}", message);
}