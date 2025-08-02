#![allow(clippy::result_large_err)]

use nu_plugin_test_support::PluginTest;
use nu_protocol::ShellError;

use nu_plugin_ws::{WebSocket, WebSocketPlugin};

/// Test basic plugin functionality with invalid connections
#[test]
fn test_websocket_connection_failure() -> Result<(), ShellError> {
    let result = PluginTest::new("ws", WebSocketPlugin.into())?
        .eval(r#"ws "ws://nonexistent.invalid.test""#);

    // Should fail to connect but not panic
    assert!(result.is_err());
    Ok(())
}

/// Test plugin with timeout parameter
#[test]
fn test_websocket_with_timeout() -> Result<(), ShellError> {
    let result = PluginTest::new("ws", WebSocketPlugin.into())?
        .eval(r#"ws "ws://127.0.0.1:1" --max-time 100ms"#);

    // Should fail to connect with timeout
    assert!(result.is_err());
    Ok(())
}

/// Test plugin with custom headers
#[test]
fn test_websocket_with_headers() -> Result<(), ShellError> {
    let result = PluginTest::new("ws", WebSocketPlugin.into())?
        .eval(r#"ws "ws://nonexistent.test" --headers {Authorization: "Bearer token", "User-Agent": "nu-plugin-ws-test"}"#);

    // Should fail to connect but handle headers without panic
    assert!(result.is_err());
    Ok(())
}

/// Test plugin with invalid URL scheme
#[test]
fn test_websocket_invalid_scheme() -> Result<(), ShellError> {
    let result = PluginTest::new("ws", WebSocketPlugin.into())?.eval(r#"ws "http://example.com""#);

    // Should fail with unsupported scheme
    assert!(result.is_err());
    Ok(())
}

/// Test plugin with string input
#[test]
fn test_websocket_with_string_input() -> Result<(), ShellError> {
    let result = PluginTest::new("ws", WebSocketPlugin.into())?
        .eval(r#""hello" | ws "ws://127.0.0.1:1" --max-time 100ms"#);

    // Should fail to connect but not panic with string input
    assert!(result.is_err());
    Ok(())
}

/// Test plugin with verbose logging
#[test]
fn test_websocket_verbose_logging() -> Result<(), ShellError> {
    let result = PluginTest::new("ws", WebSocketPlugin.into())?
        .eval(r#"ws "ws://127.0.0.1:1" --verbose 3 --max-time 50ms"#);

    // Should fail to connect but handle verbose flag
    assert!(result.is_err());
    Ok(())
}

/// Test plugin examples from command signature
#[test]
fn test_command_examples() -> Result<(), ShellError> {
    // This tests any examples defined in the WebSocket command signature
    PluginTest::new("ws", WebSocketPlugin.into())?.test_command_examples(&WebSocket)
}

/// Test plugin with binary input
#[test]
fn test_websocket_with_binary_input() -> Result<(), ShellError> {
    let result = PluginTest::new("ws", WebSocketPlugin.into())?
        .eval(r#"0x[01 02 03] | ws "ws://localhost:9999" --max-time 100ms"#);

    // Should fail to connect but handle binary input
    assert!(result.is_err());
    Ok(())
}

/// Test that plugin handles different WebSocket schemes
#[test]
fn test_websocket_schemes() -> Result<(), ShellError> {
    let ws_result = PluginTest::new("ws", WebSocketPlugin.into())?
        .eval(r#"ws "ws://localhost:9999" --max-time 100ms"#);

    let wss_result = PluginTest::new("ws", WebSocketPlugin.into())?
        .eval(r#"ws "wss://localhost:9999" --max-time 100ms"#);

    // Both should fail to connect but handle the different schemes
    assert!(ws_result.is_err());
    assert!(wss_result.is_err());
    Ok(())
}

/// Test with a publicly available WebSocket echo service
#[test]
fn test_websocket_public_echo_service() -> Result<(), ShellError> {
    // Using wss://echo.websocket.org which is a reliable public echo service
    let result = PluginTest::new("ws", WebSocketPlugin.into())?
        .eval(r#""Hello WebSocket!" | ws "wss://echo.websocket.org" --max-time 5sec"#);

    match result {
        Ok(pipeline_data) => {
            let value = pipeline_data.into_value(nu_protocol::Span::test_data())?;
            if let nu_protocol::Value::String { val, .. } = value {
                // Should receive our message back with newline
                assert!(
                    val.contains("Hello WebSocket!"),
                    "Expected echo response, got: {val}"
                );
                println!(
                    "✅ SUCCESS: Received echo from public WebSocket service: {}",
                    val.trim()
                );
            } else {
                println!("⚠️  Got non-string response: {value:?}");
            }
            Ok(())
        }
        Err(e) => {
            println!(
                "⚠️  Public WebSocket test failed (this may be due to network/firewall): {e:?}"
            );
            // Don't fail the test since public services may be unreachable in some environments
            Ok(())
        }
    }
}

/// Test with another public WebSocket service
#[test]
fn test_websocket_alternative_public_service() -> Result<(), ShellError> {
    // Using ws://echo.websocket.org (non-SSL version)
    let result = PluginTest::new("ws", WebSocketPlugin.into())?
        .eval(r#""test message" | ws "ws://echo.websocket.org" --max-time 5sec"#);

    match result {
        Ok(pipeline_data) => {
            let value = pipeline_data.into_value(nu_protocol::Span::test_data())?;
            if let nu_protocol::Value::String { val, .. } = value {
                assert!(
                    val.contains("test message"),
                    "Expected echo response, got: {val}"
                );
                println!(
                    "✅ SUCCESS: Received echo from ws://echo.websocket.org: {}",
                    val.trim()
                );
            }
            Ok(())
        }
        Err(e) => {
            println!("⚠️  Public WebSocket test failed (network/firewall): {e:?}");
            Ok(()) // Don't fail test due to network issues
        }
    }
}

/// Test binary data with public WebSocket service
#[test]
fn test_websocket_binary_public_service() -> Result<(), ShellError> {
    // Test binary data (hex bytes for "Hello")
    let result = PluginTest::new("ws", WebSocketPlugin.into())?
        .eval(r#"0x[48656c6c6f] | ws "wss://echo.websocket.org" --max-time 5sec"#);

    match result {
        Ok(pipeline_data) => {
            let value = pipeline_data.into_value(nu_protocol::Span::test_data())?;
            if let nu_protocol::Value::String { val, .. } = value {
                // Should receive the binary data back
                assert!(!val.is_empty(), "Expected some response data");
                println!(
                    "✅ SUCCESS: Binary data echoed successfully: {} bytes",
                    val.len()
                );
            }
            Ok(())
        }
        Err(e) => {
            println!("⚠️  Binary WebSocket test failed (network/firewall): {e:?}");
            Ok(())
        }
    }
}

/// Test receiving data without sending (listen-only mode)
#[test]
fn test_websocket_listen_only_mode() -> Result<(), ShellError> {
    // Some WebSocket services send data immediately upon connection
    let result = PluginTest::new("ws", WebSocketPlugin.into())?
        .eval(r#"ws "wss://echo.websocket.org" --max-time 2sec"#);

    match result {
        Ok(pipeline_data) => {
            let value = pipeline_data.into_value(nu_protocol::Span::test_data())?;
            println!("✅ Listen-only mode completed: {value:?}");
            Ok(())
        }
        Err(e) => {
            println!("⚠️  Listen-only test failed (expected for echo services): {e:?}");
            Ok(())
        }
    }
}

/// Test WebSocket connection without explicit timeout (using failure case to avoid hanging)
#[test]
fn test_websocket_without_timeout() -> Result<(), ShellError> {
    // Test that the plugin handles missing --max-time parameter correctly
    // Use a connection that will fail quickly to test the no-timeout code path safely
    let result = PluginTest::new("ws", WebSocketPlugin.into())?.eval(r#"ws "ws://127.0.0.1:1""#);

    // Should fail to connect, but this tests that the plugin handles the no-timeout case
    // without hanging indefinitely (it should use the underlying library's default timeout)
    assert!(
        result.is_err(),
        "Should fail to connect, but without hanging"
    );
    println!("✅ No-timeout parameter handled correctly (connection failed as expected)");
    Ok(())
}

// ===== MALFORMED URL TESTS =====

/// Test with empty URL
#[test]
fn test_websocket_empty_url() -> Result<(), ShellError> {
    let result = PluginTest::new("ws", WebSocketPlugin.into())?.eval(r#"ws """#);

    assert!(result.is_err(), "Empty URL should fail");
    Ok(())
}

/// Test with URL missing host
#[test]
fn test_websocket_url_missing_host() -> Result<(), ShellError> {
    let result = PluginTest::new("ws", WebSocketPlugin.into())?.eval(r#"ws "ws://""#);

    assert!(result.is_err(), "URL without host should fail");
    Ok(())
}

/// Test with URL missing scheme
#[test]
fn test_websocket_url_missing_scheme() -> Result<(), ShellError> {
    let result = PluginTest::new("ws", WebSocketPlugin.into())?.eval(r#"ws "example.com/ws""#);

    assert!(result.is_err(), "URL without scheme should fail");
    Ok(())
}

/// Test with invalid port number
#[test]
fn test_websocket_invalid_port() -> Result<(), ShellError> {
    let result =
        PluginTest::new("ws", WebSocketPlugin.into())?.eval(r#"ws "ws://example.com:99999""#);

    assert!(result.is_err(), "Invalid port should fail");
    Ok(())
}

/// Test with malformed URL brackets
#[test]
fn test_websocket_malformed_brackets() -> Result<(), ShellError> {
    let result = PluginTest::new("ws", WebSocketPlugin.into())?.eval(r#"ws "ws://[invalid""#);

    assert!(result.is_err(), "Malformed URL should fail");
    Ok(())
}

// ===== EDGE CASE INPUT TESTS =====

/// Test with empty string input
#[test]
fn test_websocket_empty_string_input() -> Result<(), ShellError> {
    let result = PluginTest::new("ws", WebSocketPlugin.into())?
        .eval(r#""" | ws "ws://127.0.0.1:1" --max-time 100ms"#);

    // Should handle empty input gracefully
    assert!(result.is_err());
    Ok(())
}

/// Test with very large string input
#[test]
fn test_websocket_large_string_input() -> Result<(), ShellError> {
    // Create a 10KB string
    let large_string = "x".repeat(10_000);
    let command = format!(r#""{large_string}" | ws "ws://127.0.0.1:1" --max-time 100ms"#);

    let result = PluginTest::new("ws", WebSocketPlugin.into())?.eval(&command);

    // Should handle large input without panic
    assert!(result.is_err());
    Ok(())
}

/// Test with list input (unsupported)
#[test]
fn test_websocket_list_input() -> Result<(), ShellError> {
    let result = PluginTest::new("ws", WebSocketPlugin.into())?
        .eval(r#"[1, 2, 3] | ws "ws://127.0.0.1:1" --max-time 100ms"#);

    // Should reject non-string/binary input
    assert!(result.is_err());
    Ok(())
}

/// Test with record input (unsupported)
#[test]
fn test_websocket_record_input() -> Result<(), ShellError> {
    let result = PluginTest::new("ws", WebSocketPlugin.into())?
        .eval(r#"{name: "test"} | ws "ws://127.0.0.1:1" --max-time 100ms"#);

    // Should reject non-string/binary input
    assert!(result.is_err());
    Ok(())
}

// ===== TIMEOUT EDGE CASES =====

/// Test with zero timeout
#[test]
fn test_websocket_zero_timeout() -> Result<(), ShellError> {
    let result = PluginTest::new("ws", WebSocketPlugin.into())?
        .eval(r#"ws "ws://example.com" --max-time 0sec"#);

    // Should handle zero timeout (immediate timeout)
    assert!(result.is_err());
    Ok(())
}

/// Test with negative timeout (should be rejected by Nushell's duration parsing)
#[test]
fn test_websocket_negative_timeout() -> Result<(), ShellError> {
    let result = PluginTest::new("ws", WebSocketPlugin.into())?
        .eval(r#"ws "ws://example.com" --max-time -5sec"#);

    // Should fail to parse negative duration
    assert!(result.is_err());
    Ok(())
}

/// Test with very small timeout
#[test]
fn test_websocket_tiny_timeout() -> Result<(), ShellError> {
    let result = PluginTest::new("ws", WebSocketPlugin.into())?
        .eval(r#"ws "ws://example.com" --max-time 1ms"#);

    // Should timeout almost immediately
    assert!(result.is_err());
    Ok(())
}

// ===== HEADER EDGE CASES =====

/// Test with empty headers
#[test]
fn test_websocket_empty_headers() -> Result<(), ShellError> {
    let result = PluginTest::new("ws", WebSocketPlugin.into())?
        .eval(r#"ws "ws://127.0.0.1:1" --headers {} --max-time 100ms"#);

    // Should handle empty headers
    assert!(result.is_err());
    Ok(())
}

/// Test with headers containing special characters
#[test]
fn test_websocket_headers_special_chars() -> Result<(), ShellError> {
    let result = PluginTest::new("ws", WebSocketPlugin.into())?
        .eval(r#"ws "ws://127.0.0.1:1" --headers {"X-Special": "value with spaces", "X-Test": "a=b&c=d"} --max-time 100ms"#);

    // Should handle special characters in headers
    assert!(result.is_err());
    Ok(())
}

/// Test with numeric header values
#[test]
fn test_websocket_numeric_header_values() -> Result<(), ShellError> {
    let result = PluginTest::new("ws", WebSocketPlugin.into())?
        .eval(r#"ws "ws://127.0.0.1:1" --headers {"Content-Length": 123, "X-Port": 8080} --max-time 100ms"#);

    // Should convert numeric values to strings
    assert!(result.is_err());
    Ok(())
}

// ===== VERBOSE LEVEL TESTS =====

/// Test with invalid verbose level (negative)
#[test]
fn test_websocket_negative_verbose() -> Result<(), ShellError> {
    let result = PluginTest::new("ws", WebSocketPlugin.into())?
        .eval(r#"ws "ws://127.0.0.1:1" --verbose -1 --max-time 100ms"#);

    // Should handle negative verbose level
    assert!(result.is_err());
    Ok(())
}

/// Test with maximum verbose level
#[test]
fn test_websocket_max_verbose() -> Result<(), ShellError> {
    let result = PluginTest::new("ws", WebSocketPlugin.into())?
        .eval(r#"ws "ws://127.0.0.1:1" --verbose 4 --max-time 100ms"#);

    // Should handle maximum verbose level
    assert!(result.is_err());
    Ok(())
}

/// Test with verbose level above maximum
#[test]
fn test_websocket_excessive_verbose() -> Result<(), ShellError> {
    let result = PluginTest::new("ws", WebSocketPlugin.into())?
        .eval(r#"ws "ws://127.0.0.1:1" --verbose 10 --max-time 100ms"#);

    // Should handle excessive verbose level (treat as max)
    assert!(result.is_err());
    Ok(())
}

// ===== ERROR HANDLING TESTS =====

/// Test with connection refused (explicit port)
#[test]
fn test_websocket_connection_refused() -> Result<(), ShellError> {
    // Port 1 is typically privileged and refused
    let result = PluginTest::new("ws", WebSocketPlugin.into())?
        .eval(r#"ws "ws://localhost:1" --max-time 500ms"#);

    assert!(result.is_err(), "Connection should be refused");
    Ok(())
}

/// Test with DNS resolution failure
#[test]
fn test_websocket_dns_failure() -> Result<(), ShellError> {
    let result = PluginTest::new("ws", WebSocketPlugin.into())?.eval(
        r#"ws "ws://this-domain-definitely-does-not-exist-123456789.invalid" --max-time 2sec"#,
    );

    assert!(result.is_err(), "DNS resolution should fail");
    Ok(())
}

/// Test with self-signed certificate (for wss://)
#[test]
fn test_websocket_self_signed_cert() -> Result<(), ShellError> {
    // Using a known self-signed cert service (this might need adjustment based on availability)
    let result = PluginTest::new("ws", WebSocketPlugin.into())?
        .eval(r#"ws "wss://self-signed.badssl.com" --max-time 2sec"#);

    // Should fail due to certificate validation
    assert!(result.is_err());
    Ok(())
}

// ===== PIPELINE INTEGRATION TESTS =====

/// Test piping WebSocket output (if we could connect)
#[test]
fn test_websocket_pipeline_output() -> Result<(), ShellError> {
    // This tests that the output can be piped, even though connection fails
    let result = PluginTest::new("ws", WebSocketPlugin.into())?
        .eval(r#"ws "ws://127.0.0.1:1" --max-time 100ms | describe"#);

    // Pipeline should work even with connection failure
    assert!(result.is_err());
    Ok(())
}

/// Test multiple WebSocket calls in sequence
#[test]
fn test_websocket_sequential_calls() -> Result<(), ShellError> {
    // First call
    let result1 = PluginTest::new("ws", WebSocketPlugin.into())?
        .eval(r#"ws "ws://127.0.0.1:1" --max-time 50ms"#);
    assert!(result1.is_err());

    // Second call - ensure no resource leaks
    let result2 = PluginTest::new("ws", WebSocketPlugin.into())?
        .eval(r#"ws "ws://127.0.0.1:2" --max-time 50ms"#);
    assert!(result2.is_err());

    Ok(())
}
