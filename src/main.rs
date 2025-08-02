use nu_plugin::{serve_plugin, JsonSerializer};
use nu_plugin_ws::WebSocketPlugin;

fn main() {
    serve_plugin(&WebSocketPlugin, JsonSerializer)
}
