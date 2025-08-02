use nu_plugin::EvaluatedCall;
use nu_protocol::{ShellError, Signals, Span, Value};
use url::Url;

#[cfg(not(target_arch = "wasm32"))]
use std::collections::HashMap;
use std::{
    collections::VecDeque,
    io::Read,
    sync::{
        mpsc::{self, Receiver, RecvTimeoutError},
        Arc, Mutex,
    },
    thread,
    time::{Duration, Instant},
};
use tungstenite::ClientRequestBuilder;

type WebSocketConnection =
    Arc<Mutex<tungstenite::WebSocket<tungstenite::stream::MaybeTlsStream<std::net::TcpStream>>>>;

pub struct WebSocketClient {
    rx: Arc<Mutex<Receiver<Vec<u8>>>>,
    deadline: Option<Instant>,
    buf_deque: VecDeque<u8>,
    signals: Signals,
    span: Span,
}

impl WebSocketClient {
    pub fn new(
        rx: Receiver<Vec<u8>>,
        timeout: Option<Duration>,
        signals: Signals,
        span: Span,
    ) -> Self {
        let mut client = Self {
            rx: Arc::new(Mutex::new(rx)),
            deadline: None,
            buf_deque: VecDeque::new(),
            signals,
            span,
        };
        if let Some(timeout) = timeout {
            client.deadline = Some(Instant::now() + timeout);
        }
        client
    }
}

impl Read for WebSocketClient {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        // If we have data in the buffer, return it immediately
        if !self.buf_deque.is_empty() {
            let mut len = 0;
            for buf_slot in buf {
                if let Some(b) = self.buf_deque.pop_front() {
                    *buf_slot = b;
                    len += 1;
                } else {
                    break;
                }
            }
            return Ok(len);
        }

        let rx = self.rx.lock().expect("Could not get lock on receiver");
        let poll_interval = Duration::from_millis(100);

        // Poll for new data with regular signal checking
        loop {
            // Check for signals (Ctrl+C) before each poll
            if let Err(e) = self.signals.check(&self.span) {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Interrupted,
                    e.to_string(),
                ));
            }

            // Determine how long to wait this iteration
            let wait_time = match self.deadline {
                Some(deadline) => {
                    match deadline.checked_duration_since(Instant::now()) {
                        Some(remaining) => {
                            // Use the smaller of remaining time or poll interval
                            remaining.min(poll_interval)
                        }
                        None => {
                            // Deadline has already passed
                            return Ok(0);
                        }
                    }
                }
                None => poll_interval, // No deadline, just use poll interval
            };

            // Poll for data with timeout
            match rx.recv_timeout(wait_time) {
                Ok(bytes) => {
                    // Got data! Add to buffer and return it
                    for b in bytes {
                        self.buf_deque.push_back(b);
                    }

                    // Return as much data as fits in the provided buffer
                    let mut len = 0;
                    for buf_slot in buf {
                        if let Some(b) = self.buf_deque.pop_front() {
                            *buf_slot = b;
                            len += 1;
                        } else {
                            break;
                        }
                    }
                    return Ok(len);
                }
                Err(RecvTimeoutError::Timeout) => {
                    // No data available right now, continue loop to check signals again
                    continue;
                }
                Err(RecvTimeoutError::Disconnected) => {
                    // Channel disconnected - real EOF
                    return Ok(0);
                }
            }
        }
    }
}

pub fn connect(
    url: Url,
    timeout: Option<Duration>,
    headers: HashMap<String, String>,
    signals: Signals,
    span: Span,
) -> Option<(WebSocketClient, WebSocketConnection)> {
    log::trace!("Building WebSocket request for: {}", url);

    let mut builder = ClientRequestBuilder::new(url.as_str().parse().ok()?);
    let origin = format!(
        "{}://{}:{}",
        url.scheme(),
        url.host_str().unwrap_or_default(),
        url.port().unwrap_or_default()
    );

    log::trace!("Setting Origin header to: {}", origin);

    builder = builder.with_header("Origin", origin);

    for (k, v) in headers {
        log::trace!("Adding header: {} = {}", k, v);
        builder = builder.with_header(k, v);
    }

    log::debug!("Attempting WebSocket connection...");

    match tungstenite::connect(builder) {
        Ok((websocket, _)) => {
            log::debug!("WebSocket handshake completed successfully");

            let (tx_read, rx_read) = mpsc::sync_channel(1024);

            log::trace!("Created channel for reader communication");

            let tx_read = Arc::new(tx_read);
            let websocket = Arc::new(Mutex::new(websocket));

            // Thread for reading from websocket
            let ws_clone = websocket.clone();
            thread::Builder::new()
                .name("websocket reader".to_string())
                .spawn(move || {
                    log::debug!("WebSocket reader thread started");
                    loop {
                        let tx_read = tx_read.clone();
                        let mut ws = ws_clone.lock().unwrap();
                        match ws.read() {
                            Ok(msg) => match msg {
                                tungstenite::Message::Text(msg) => {
                                    log::debug!("Received Text message: {} bytes", msg.len());
                                    log::trace!("Text content: {:?}", msg);
                                    // Add newline after each WebSocket message for proper line separation
                                    let mut data = msg.into_bytes();
                                    data.push(b'\n');
                                    if tx_read.send(data).is_err() {
                                        log::debug!("Channel closed, closing WebSocket");
                                        ws.close(Some(tungstenite::protocol::CloseFrame{
                                            code: tungstenite::protocol::frame::coding::CloseCode::Normal,
                                            reason: std::borrow::Cow::Borrowed("byte stream closed"),
                                        })).expect("Could not close connection");
                                        return;
                                    }
                                    log::trace!("Message sent to channel successfully, continuing to read...");
                                }
                                tungstenite::Message::Binary(msg) => {
                                    log::debug!("Received Binary message: {} bytes", msg.len());
                                    // Add newline after each WebSocket message for proper line separation
                                    let mut data = msg;
                                    data.push(b'\n');
                                    if tx_read.send(data).is_err() {
                                        log::debug!("Channel closed, closing WebSocket");
                                        ws.close(Some(tungstenite::protocol::CloseFrame{
                                            code: tungstenite::protocol::frame::coding::CloseCode::Normal,
                                            reason: std::borrow::Cow::Borrowed("byte stream closed"),
                                        })).expect("Could not close connection");
                                        return;
                                    }
                                }
                                tungstenite::Message::Close(..) => {
                                    log::debug!("Received Close message");
                                    drop(tx_read);
                                    return;
                                }
                                _ => {
                                    log::trace!("Received other message type: {:?}", msg);
                                    continue;
                                }
                            },
                            Err(e) => {
                                log::error!("WebSocket read error: {:?}", e);
                                log::debug!("WebSocket reader thread exiting due to error");
                                drop(tx_read);
                                return;
                            }
                        }
                    }
                })
                .ok()?;

            log::trace!("Created WebSocketClient, connection ready");

            Some((
                WebSocketClient::new(rx_read, timeout, signals, span),
                websocket,
            ))
        }
        Err(e) => {
            log::error!("Failed to connect to WebSocket: {:?}", e);
            None
        }
    }
}

#[allow(clippy::result_large_err)]
pub fn http_parse_url(
    call: &EvaluatedCall,
    span: Span,
    raw_url: Value,
) -> Result<(String, Url), ShellError> {
    let requested_url = raw_url.coerce_into_string()?;
    let url = match Url::parse(&requested_url) {
        Ok(u) => u,
        Err(_e) => {
            return Err(ShellError::UnsupportedInput { msg: "Incomplete or incorrect URL. Expected a full URL, e.g., https://www.example.com"
                    .to_string(), input: format!("value: '{requested_url:?}'"), msg_span: call.head, input_span: span });
        }
    };

    Ok((requested_url, url))
}

#[allow(clippy::result_large_err)]
pub fn request_headers(headers: Option<Value>) -> Result<HashMap<String, String>, ShellError> {
    let mut custom_headers: HashMap<String, Value> = HashMap::new();

    if let Some(headers) = headers {
        match &headers {
            Value::Record { val, .. } => {
                for (k, v) in &**val {
                    custom_headers.insert(k.to_string(), v.clone());
                }
            }

            Value::List { vals: table, .. } => {
                if table.len() == 1 {
                    // single row([key1 key2]; [val1 val2])
                    match &table[0] {
                        Value::Record { val, .. } => {
                            for (k, v) in &**val {
                                custom_headers.insert(k.to_string(), v.clone());
                            }
                        }

                        x => {
                            return Err(ShellError::CantConvert {
                                to_type: "string list or single row".into(),
                                from_type: x.get_type().to_string(),
                                span: headers.span(),
                                help: None,
                            });
                        }
                    }
                } else {
                    // primitive values ([key1 val1 key2 val2])
                    for row in table.chunks(2) {
                        if row.len() == 2 {
                            custom_headers.insert(row[0].coerce_string()?, row[1].clone());
                        }
                    }
                }
            }

            x => {
                return Err(ShellError::CantConvert {
                    to_type: "string list or single row".into(),
                    from_type: x.get_type().to_string(),
                    span: headers.span(),
                    help: None,
                });
            }
        };
    }

    let mut result = HashMap::new();
    for (k, v) in custom_headers {
        if let Ok(s) = v.coerce_into_string() {
            result.insert(k, s);
        }
    }

    Ok(result)
}
