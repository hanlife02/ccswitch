# CCSwitch

A command-line tool for automatic switching between multiple model API channels written in Rust.

## Features

- **Multi-channel support**: Configure multiple API endpoints for the same model
- **Automatic failover**: Automatically switches to available channels when one fails
- **Health checking**: Test channel availability before making requests
- **Easy management**: Add, remove, and list channels with simple commands
- **Model agnostic**: Works with OpenAI, Claude, and other compatible APIs

## Installation

```bash
cargo install ccswitch
```

Or clone and build from source:

```bash
git clone https://github.com/yourusername/ccswitch
cd ccswitch
cargo build --release
```

## Configuration

CCSwitch stores its configuration in your system's config directory:
- Linux: `~/.config/ccswitch/config.json`
- macOS: `~/Library/Application Support/ccswitch/config.json`
- Windows: `%APPDATA%\ccswitch\config.json`

## Usage

### Add a new channel

```bash
# Add OpenAI channel
ccswitch add openai https://api.openai.com/v1/chat/completions -k YOUR_API_KEY -m gpt-3.5-turbo

# Add Anthropic Claude channel
ccswitch add claude https://api.anthropic.com/v1/messages -k YOUR_API_KEY -m claude-3-sonnet-20240229

# Add custom endpoint
ccswitch add custom https://your-api.com/v1/chat -k YOUR_API_KEY
```

### List all channels

```bash
ccswitch list
```

### Test channel availability

```bash
# Test all channels
ccswitch test

# Test specific channel
ccswitch test openai
```

### Make a request with automatic switching

```bash
# Simple request
ccswitch request "Hello, how are you?"

# Request with specific model
ccswitch request "Explain quantum computing" -m gpt-4

# Request with custom parameters
ccswitch request "Write a story" -m claude-3-sonnet-20240229 --max-tokens 500 -t 0.8
```

### Remove a channel

```bash
ccswitch remove openai
```

## How it works

1. When you make a request, CCSwitch finds all channels that support the requested model
2. It tests channels in priority order (configurable)
3. The first available channel is used to make the actual API request
4. If a channel fails, it automatically tries the next available one
5. The response includes information about which channel was used

## Configuration Format

The config file uses JSON format:

```json
{
  "channels": {
    "openai": {
      "name": "openai",
      "url": "https://api.openai.com/v1/chat/completions",
      "api_key": "sk-...",
      "model": "gpt-3.5-turbo",
      "enabled": true,
      "priority": 0
    }
  },
  "default_model": "gpt-3.5-turbo",
  "timeout_seconds": 30,
  "retry_attempts": 3
}
```

## License

MIT OR Apache-2.0