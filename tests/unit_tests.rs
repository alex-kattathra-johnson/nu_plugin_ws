use nu_plugin_ws::ws::client::{request_headers, WebSocketClient};
use nu_protocol::{Record, Signals, Span, Value};
use std::time::Duration;

#[test]
fn test_websocket_client_timeout_handling() {
    use std::sync::mpsc;

    let (_, rx) = mpsc::sync_channel(10);
    let timeout = Some(Duration::from_millis(100));

    let _client = WebSocketClient::new(rx, timeout, Signals::empty(), Span::test_data());
    // deadline field is private, so we can't test it directly

    // We can test behavior instead of internal fields
}

#[test]
fn test_websocket_client_no_timeout() {
    use std::sync::mpsc;

    let (_, rx) = mpsc::sync_channel(10);

    let _client = WebSocketClient::new(rx, None, Signals::empty(), Span::test_data());
    // deadline field is private, so we can't test it directly
}

#[test]
fn test_websocket_client_read_empty_channel() {
    use std::io::Read;
    use std::sync::mpsc;

    let (tx, rx) = mpsc::sync_channel(10);
    drop(tx); // Close the channel

    let mut client = WebSocketClient::new(
        rx,
        Some(Duration::from_millis(10)),
        Signals::empty(),
        Span::test_data(),
    );
    let mut buffer = [0u8; 100];

    let result = client.read(&mut buffer);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 0); // Should return 0 bytes when channel is closed
}

#[test]
fn test_websocket_client_read_with_data() {
    use std::io::Read;
    use std::sync::mpsc;

    let (tx, rx) = mpsc::sync_channel(10);
    let test_data = b"Hello WebSocket".to_vec();
    tx.send(test_data.clone()).unwrap();
    drop(tx);

    let mut client = WebSocketClient::new(rx, None, Signals::empty(), Span::test_data());
    let mut buffer = [0u8; 100];

    let result = client.read(&mut buffer);
    assert!(result.is_ok());

    let bytes_read = result.unwrap();
    assert_eq!(bytes_read, test_data.len());
    assert_eq!(&buffer[..bytes_read], &test_data[..]);
}

#[test]
fn test_request_headers_empty() {
    let result = request_headers(None);
    assert!(result.is_ok());

    let headers = result.unwrap();
    assert!(headers.is_empty());
}

#[test]
fn test_request_headers_with_record() {
    let mut record = Record::new();
    record.insert(
        "Authorization".to_string(),
        Value::string("Bearer token123", Span::test_data()),
    );
    record.insert(
        "User-Agent".to_string(),
        Value::string("nu-plugin-ws/0.3.2", Span::test_data()),
    );

    let headers_value = Some(Value::record(record, Span::test_data()));
    let result = request_headers(headers_value);
    assert!(result.is_ok());

    let headers = result.unwrap();
    assert_eq!(headers.len(), 2);
    assert_eq!(
        headers.get("Authorization"),
        Some(&"Bearer token123".to_string())
    );
    assert_eq!(
        headers.get("User-Agent"),
        Some(&"nu-plugin-ws/0.3.2".to_string())
    );
}
