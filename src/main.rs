use std::time::Duration;

use nu_plugin::{EvaluatedCall, JsonSerializer, serve_plugin};
use nu_plugin::{EngineInterface, Plugin, PluginCommand};
use nu_protocol::{ByteStream, ByteStreamType, Category, LabeledError, PipelineData, Signature, SyntaxShape, Type, Value};

mod ws;
use ws::client::{connect, http_parse_url, request_headers};

struct WebSocketPlugin;

impl Plugin for WebSocketPlugin {
    fn version(&self) -> String {
        env!("CARGO_PKG_VERSION").into()
    }

    fn commands(&self) -> Vec<Box<dyn PluginCommand<Plugin = Self>>> {
        vec![
            Box::new(WebSocket),
        ]
    }
}

struct WebSocket;

impl PluginCommand for WebSocket {
    type Plugin = WebSocketPlugin;

    fn name(&self) -> &str {
        "ws"
    }

    fn description(&self) -> &str {
        "streams output from a websocket"
    }

    fn signature(&self) -> Signature {
        Signature::build(PluginCommand::name(self))
            .input_output_type(Type::String, Type::Int)
            .required(
                "URL",
                SyntaxShape::String,
                "The URL to stream from (ws:// or wss://).",
            )
            .named(
                "headers",
                SyntaxShape::Any,
                "custom headers you want to add ",
                Some('H'),
            )
            .named(
                "max-time",
                SyntaxShape::Duration,
                "max duration before timeout occurs",
                Some('m'),
            )
            .filter()
            .category(Category::Network)
    }

    fn run(
        &self,
        _plugin: &Self::Plugin,
        engine: &EngineInterface,
        call: &EvaluatedCall,
        _input: PipelineData,
    ) -> Result<PipelineData, LabeledError> {
        let url: Value = call.req(0)?;
        let headers: Option<Value> = call.get_flag("headers")?;
        let timeout: Option<Value> = call.get_flag("max-time")?;

        let span = url.span();

        let (_, requested_url) = http_parse_url(call, span, url)?;

        if ["ws", "wss"].contains(&requested_url.scheme()) {
            let timeout = timeout.map(|ref val| {
                Duration::from_nanos(
                    val.as_duration()
                        .expect("Timeout should be set to duration") as u64,
                )
            });
            if let Some(cr) = connect(
                requested_url,
                timeout,
                request_headers(headers)?,
            ) {
                let reader = Box::new(cr);
                return Ok(PipelineData::ByteStream(
                    ByteStream::read(
                        reader,
                        span,
                        engine.signals().clone(),
                        ByteStreamType::Unknown,
                    ),
                    None,
                ));
            }
        }

        Err(LabeledError::new("Unsupported input for command"))
    }
}

fn main() {
    serve_plugin(&WebSocketPlugin, JsonSerializer)
}
