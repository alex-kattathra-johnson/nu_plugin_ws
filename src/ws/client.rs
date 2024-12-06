use nu_plugin::EvaluatedCall;
use nu_protocol::{ShellError, Span, Value};
use url::Url;

#[cfg(not(target_arch = "wasm32"))]
use std::collections::HashMap;
use std::{
    collections::VecDeque,
    io::Read,
    sync::{
        mpsc::{self, Receiver, TryRecvError},
        Arc, Mutex,
    },
    thread,
    time::{Duration, Instant},
};
use tungstenite::ClientRequestBuilder;

pub struct ChannelReader {
    rx: Arc<Mutex<Receiver<Vec<u8>>>>,
    deadline: Option<Instant>,
    buf_deque: VecDeque<u8>,
}

impl ChannelReader {
    pub fn new(rx: Receiver<Vec<u8>>, timeout: Option<Duration>) -> Self {
        let mut cr = Self {
            rx: Arc::new(Mutex::new(rx)),
            deadline: None,
            buf_deque: VecDeque::new(),
        };
        if let Some(timeout) = timeout {
            cr.deadline = Some(Instant::now() + timeout);
        }
        cr
    }
}

impl Read for ChannelReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let rx = self.rx.lock().expect("Could not get lock on receiver");

        let bytes = match self.deadline {
            Some(deadline) => rx
                .recv_timeout(deadline.duration_since(Instant::now()))
                .map_err(|_| TryRecvError::Disconnected),
            None => rx.recv().map_err(|_| TryRecvError::Disconnected),
        };

        let bytes = match bytes {
            Ok(bytes) => bytes,
            Err(..) => return Ok(0),
        };

        for b in bytes {
            self.buf_deque.push_back(b);
        }

        let mut len = 0;
        for buf in buf {
            if let Some(b) = self.buf_deque.pop_front() {
                *buf = b;
                len += 1;
            } else {
                break;
            }
        }
        Ok(len)
    }
}

pub fn connect(
    url: Url,
    timeout: Option<Duration>,
    headers: HashMap<String, String>,
) -> Option<ChannelReader> {
    let mut builder = ClientRequestBuilder::new(url.as_str().parse().ok()?);
    builder = builder.with_header(
        "Origin",
        format!(
            "{}://{}:{}",
            url.scheme(),
            url.host_str().unwrap_or_default(),
            url.port().unwrap_or_default()
        ),
    );
    for (k, v) in headers {
        builder = builder.with_header(k, v);
    }
    match tungstenite::connect(builder) {
        Ok((mut websocket, _)) => {
            let (tx, rx) = mpsc::sync_channel(1024);
            let tx = Arc::new(tx);
            thread::Builder::new()
                .name("websocket response sender".to_string())
                .spawn(move || loop {
                    let tx = tx.clone();
                    match websocket.read() {
                        Ok(msg) => match msg {
                            tungstenite::Message::Text(msg) => {
                                if tx.send(msg.as_bytes().to_vec()).is_err() {
                                    websocket.close(Some(tungstenite::protocol::CloseFrame{
                                    code: tungstenite::protocol::frame::coding::CloseCode::Normal,
                                    reason: std::borrow::Cow::Borrowed("byte stream closed"),
                                })).expect("Could not close connection")
                                }
                            }
                            tungstenite::Message::Binary(msg) => {
                                if tx.send(msg).is_err() {
                                    websocket.close(Some(tungstenite::protocol::CloseFrame{
                                    code: tungstenite::protocol::frame::coding::CloseCode::Normal,
                                    reason: std::borrow::Cow::Borrowed("byte stream closed"),
                                })).expect("Could not close connection")
                                }
                            }
                            tungstenite::Message::Close(..) => {
                                drop(tx);
                                return;
                            }
                            _ => continue,
                        },
                        _ => {
                            drop(tx);
                            return;
                        }
                    }
                })
                .ok()?;
            Some(ChannelReader::new(rx, timeout))
        }
        Err(..) => None,
    }
}

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
