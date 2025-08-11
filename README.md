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

### Interactive WebSocket Sessions

For interactive WebSocket communication, you can use Nushell's built-in commands to create interactive workflows:

#### Method 1: Using a loop with input

Create an interactive session using Nushell's `loop` and `input` commands:

```bash
# Simple interactive loop
loop {
  let msg = input "Message (or 'quit' to exit): "
  if $msg == "quit" { break }
  $msg | ws "wss://echo.websocket.org"
}
```

#### Method 2: Using a custom function

Define a reusable function for interactive sessions:

```bash
# Add to your Nushell config
def ws-interactive [url: string] {
  print $"Connected to ($url)"
  print "Type messages to send, or 'quit' to exit"

  loop {
    let msg = input "> "
    if $msg == "quit" {
      print "Disconnected"
      break
    }
    if $msg != "" {
      let response = $msg | ws $url
      print $"Response: ($response)"
    }
  }
}

# Use it
ws-interactive "wss://echo.websocket.org"
```

#### Method 3: Reading from a file

For automated testing or scripted interactions:

```bash
# Create a messages file
echo "message1\nmessage2\nmessage3" | save messages.txt

# Send each line as a separate message
open messages.txt | lines | each { |msg|
  $msg | ws "wss://echo.websocket.org"
  sleep 1sec  # Add delay between messages if needed
}
```

#### Method 4: Using a watch file

Create a file-based interactive session:

```bash
# In one terminal, watch a file and send its contents
watch input.txt {
  open input.txt | ws "wss://echo.websocket.org"
}

# In another terminal, write messages to the file
echo "Hello WebSocket" | save -f input.txt
```

#### Method 5: Bi-directional communication with multiple connections

For scenarios requiring separate send and receive channels:

```bash
# Terminal 1: Listen for messages
ws "wss://echo.websocket.org" | save -a responses.log

# Terminal 2: Send messages
loop {
  let msg = input "> "
  if $msg == "quit" { break }
  $msg | ws "wss://echo.websocket.org"
}
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
