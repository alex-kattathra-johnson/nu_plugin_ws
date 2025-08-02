use std::time::Duration;

use nu_plugin::{EngineInterface, EvaluatedCall, Plugin, PluginCommand};
use nu_protocol::{
    ByteStream, ByteStreamType, Category, LabeledError, PipelineData, Signature, SyntaxShape, Type,
    Value,
};

pub mod ws;
use ws::client::{connect, http_parse_url, request_headers};

pub struct WebSocketPlugin;

impl Plugin for WebSocketPlugin {
    fn version(&self) -> String {
        env!("CARGO_PKG_VERSION").into()
    }

    fn commands(&self) -> Vec<Box<dyn PluginCommand<Plugin = Self>>> {
        vec![Box::new(WebSocket)]
    }
}

pub struct WebSocket;

impl PluginCommand for WebSocket {
    type Plugin = WebSocketPlugin;

    fn name(&self) -> &str {
        "ws"
    }

    fn description(&self) -> &str {
        "connect to a websocket, send optional input data, and stream output"
    }

    fn signature(&self) -> Signature {
        Signature::build(PluginCommand::name(self))
            .input_output_types(vec![
                (Type::Nothing, Type::Any),
                (Type::String, Type::Any),
                (Type::Binary, Type::Any),
            ])
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
            .named(
                "verbose",
                SyntaxShape::Int,
                "verbosity level (0=error, 1=warn, 2=info, 3=debug, 4=trace)",
                Some('v'),
            )
            .filter()
            .category(Category::Network)
    }

    fn run(
        &self,
        _plugin: &Self::Plugin,
        engine: &EngineInterface,
        call: &EvaluatedCall,
        input: PipelineData,
    ) -> Result<PipelineData, LabeledError> {
        let url: Value = call.req(0)?;
        let headers: Option<Value> = call.get_flag("headers")?;
        let timeout: Option<Value> = call.get_flag("max-time")?;
        let verbose: Option<Value> = call.get_flag("verbose")?;

        // Set up logging based on verbose level
        let log_level_filter = if let Some(Value::Int { val, .. }) = verbose {
            match val {
                0 => log::LevelFilter::Error,
                1 => log::LevelFilter::Warn,
                2 => log::LevelFilter::Info,
                3 => log::LevelFilter::Debug,
                4 => log::LevelFilter::Trace,
                _ => log::LevelFilter::Info,
            }
        } else {
            log::LevelFilter::Error // Default to error only
        };

        // Initialize env_logger with the specified level (only if not already initialized)
        let _ = env_logger::Builder::from_default_env()
            .filter_level(log_level_filter)
            .try_init();

        let span = url.span();

        log::trace!("Parsing URL and starting WebSocket connection");

        let (_, requested_url) = http_parse_url(call, span, url)?;

        log::debug!("Connecting to: {requested_url}");

        if ["ws", "wss"].contains(&requested_url.scheme()) {
            let timeout = timeout.map(|ref val| {
                let duration = Duration::from_nanos(
                    val.as_duration()
                        .expect("Timeout should be set to duration") as u64,
                );
                log::trace!("Setting timeout to: {duration:?}");
                duration
            });

            log::trace!("Calling connect function");

            if let Some((client, websocket)) = connect(
                requested_url,
                timeout,
                request_headers(headers)?,
                engine.signals().clone(),
                span,
            ) {
                log::debug!("WebSocket connection established successfully");

                // Send input data synchronously before returning ByteStream
                match input {
                    PipelineData::Value(val, ..) => {
                        let data = match val {
                            Value::String { val, .. } => {
                                log::debug!("Sending string input: {} bytes", val.len());
                                val.into_bytes()
                            }
                            Value::Binary { val, .. } => {
                                log::debug!("Sending binary input: {} bytes", val.len());
                                val
                            }
                            _ => {
                                return Err(LabeledError::new("Input must be string or binary")
                                    .with_label("Unsupported input type", span));
                            }
                        };

                        // Send message synchronously
                        let mut ws = websocket
                            .lock()
                            .map_err(|_| LabeledError::new("Failed to lock WebSocket"))?;

                        // Try to send as Text if it's valid UTF-8, otherwise send as Binary
                        let message = match String::from_utf8(data.clone()) {
                            Ok(text) => {
                                log::debug!("Sending as Text message: {text:?}");
                                tungstenite::Message::Text(text)
                            }
                            Err(_) => {
                                log::debug!("Sending as Binary message (invalid UTF-8)");
                                tungstenite::Message::Binary(data)
                            }
                        };

                        ws.send(message).map_err(|e| {
                            LabeledError::new(format!("Failed to send WebSocket message: {e}"))
                        })?;

                        log::debug!("Message sent successfully, now starting to receive");
                    }
                    PipelineData::ByteStream(stream, ..) => {
                        let data = stream
                            .into_bytes()
                            .map_err(|e| LabeledError::new(e.to_string()))?;
                        log::debug!("Sending ByteStream input: {} bytes", data.len());

                        // Send message synchronously
                        let mut ws = websocket
                            .lock()
                            .map_err(|_| LabeledError::new("Failed to lock WebSocket"))?;

                        let message = match String::from_utf8(data.clone()) {
                            Ok(text) => tungstenite::Message::Text(text),
                            Err(_) => tungstenite::Message::Binary(data),
                        };

                        ws.send(message).map_err(|e| {
                            LabeledError::new(format!("Failed to send WebSocket message: {e}"))
                        })?;

                        log::debug!(
                            "ByteStream message sent successfully, now starting to receive"
                        );
                    }
                    PipelineData::Empty => {
                        log::debug!("No input data, only receiving from WebSocket");
                        // No input data, just read from websocket
                    }
                    _ => {
                        return Err(LabeledError::new("Unsupported input type")
                            .with_label("Input must be string, binary, or nothing", span));
                    }
                }

                log::trace!("Creating ByteStream from WebSocketClient");

                let reader = Box::new(client);

                log::debug!("Returning ByteStream to Nushell pipeline");

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
