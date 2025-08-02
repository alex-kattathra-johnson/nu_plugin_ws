# nu_plugin_ws

[![Crates.io Version](https://img.shields.io/crates/v/nu_plugin_ws?color=blue)](https://crates.io/crates/nu_plugin_ws)
[![Nushell](https://img.shields.io/badge/Nushell-v0.106.1-blue)](https://nushell.sh)

A plugin for [Nushell](https://nushell.sh), a cross-platform shell and scripting language. This plugin adds support for
streaming from a websocket.

## Installation

### Cargo

Get the latest version from [crates.io](https://crates.io/crates/nu_plugin_ws) with a local install:

```bash
# Downloads and installs the plugin
cargo install nu_plugin_ws
# Registers the plugin with Nushell
plugin add ~/.cargo/bin/nu_plugin_ws
# Activates the plugin
plugin use ws
```

### Manual build

Manual builds can also be used:

```bash
# Clone the repository
git clone https://github.com/alex-kattathra-johnson/nu_plugin_ws.git
# Enter the repo folder
cd nu_plugin_ws
# Build a release version of the plugin
cargo build -r
# Registers the plugin with Nushell
plugin add target/release/nu_plugin_ws
# Activates the plugin
plugin use ws
```

## Usage

### Basic Connection

Connect to a WebSocket and stream data:

```bash
# Connect and listen only
ws "wss://echo.websocket.org"

# With timeout
ws "wss://echo.websocket.org" --max-time 10sec
```

### Sending Messages

Send text messages by piping string data:

```bash
# Send a text message
echo "Hello WebSocket" | ws "wss://echo.websocket.org"

# Send JSON data
echo '{"message": "hello", "type": "text"}' | ws "wss://localhost:8080/chat"

# Send with custom headers
echo "authenticated message" | ws "wss://api.example.com" --headers {Authorization: "Bearer token123"}
```

Send binary data:

```bash
# Send binary data (hex format)
0x[48656c6c6f] | ws "wss://echo.websocket.org"

# Send file contents as binary
open file.bin | ws "wss://echo.websocket.org"
```

### Advanced Usage

```bash
# Multiple custom headers
ws "wss://api.example.com" --headers {
  "Authorization": "Bearer token123",
  "X-Client-ID": "my-client",
  "X-Version": "1.0"
}

# With timeout and verbose logging
echo "test message" | ws "wss://echo.websocket.org" --max-time 30sec --verbose 3

# Handle special characters and Unicode
echo "Hello 🌍 测试 русский" | ws "wss://echo.websocket.org"
```

## Development

This project uses pre-commit hooks to ensure code quality. See [CONTRIBUTING.md](CONTRIBUTING.md) for setup instructions.

Quick setup:
```bash
# Install pre-commit
pip install pre-commit
# Install the git hooks
pre-commit install
```
