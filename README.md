# CCSwitch

基于 Rust 开发的多渠道 AI 模型 API 自动切换命令行工具。

## 功能特性

- **多渠道支持**: 为同一模型配置多个 API 端点
- **自动故障转移**: 当一个渠道失效时自动切换到可用渠道
- **健康检查**: 在请求前测试渠道可用性
- **便捷管理**: 通过简单命令添加、删除和列出渠道
- **模型无关**: 支持 OpenAI、Claude 和其他兼容的 API

## 安装

```bash
cargo install ccswitch
```

或者从源码构建:

```bash
git clone https://github.com/hanlife02/ccswitch
cd ccswitch
cargo build --release
```

## 配置

CCSwitch 将配置文件存储在系统配置目录中:
- Linux: `~/.config/ccswitch/config.json`
- macOS: `~/Library/Application Support/ccswitch/config.json`
- Windows: `%APPDATA%\ccswitch\config.json`

## 使用方法

### 添加新渠道

```bash
# 添加 OpenAI 渠道
ccswitch add openai https://api.openai.com/v1/chat/completions -k YOUR_API_KEY -m gpt-3.5-turbo

# 添加 Anthropic Claude 渠道
ccswitch add claude https://api.anthropic.com/v1/messages -k YOUR_API_KEY -m claude-3-sonnet-20240229

# 添加自定义端点
ccswitch add custom https://your-api.com/v1/chat -k YOUR_API_KEY
```

### 列出所有渠道

```bash
ccswitch list
```

### 测试渠道可用性

```bash
# 测试所有渠道
ccswitch test

# 测试特定渠道
ccswitch test openai
```

### 发送请求并自动切换

```bash
# 简单请求
ccswitch request "你好，你好吗？"

# 指定模型的请求
ccswitch request "解释一下量子计算" -m gpt-4

# 自定义参数的请求
ccswitch request "写一个故事" -m claude-3-sonnet-20240229 --max-tokens 500 -t 0.8
```

### 删除渠道

```bash
ccswitch remove openai
```

## 工作原理

1. 发送请求时，CCSwitch 查找支持所需模型的所有渠道
2. 按优先级顺序测试渠道（可配置）
3. 使用第一个可用渠道进行实际 API 请求
4. 如果渠道失败，自动尝试下一个可用渠道
5. 响应中包含使用了哪个渠道的信息

## 配置文件格式

配置文件使用 JSON 格式:

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

## 许可证

MIT OR Apache-2.0