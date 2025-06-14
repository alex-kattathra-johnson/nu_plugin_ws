# nu_plugin_ws

[![Crates.io Version](https://img.shields.io/crates/v/nu_plugin_ws?color=blue)](https://crates.io/crates/nu_plugin_ws)
[![Nushell](https://img.shields.io/badge/Nushell-v0.105.1-blue)](https://nushell.sh)

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
