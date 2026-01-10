//! WebSocket API implementation.

use boa_engine::{
    Context, JsArgs, JsNativeError, JsResult, JsValue, NativeFunction,
    js_string,
    object::ObjectInitializer,
    property::Attribute,
};
use std::collections::VecDeque;
use std::sync::Arc;
use parking_lot::RwLock;

/// WebSocket ready states.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReadyState {
    Connecting = 0,
    Open = 1,
    Closing = 2,
    Closed = 3,
}

/// WebSocket implementation.
pub struct WebSocket {
    /// WebSocket URL.
    url: String,
    /// Current ready state.
    ready_state: ReadyState,
    /// Buffered amount of data.
    buffered_amount: u64,
    /// Protocol selected by server.
    protocol: String,
    /// Binary type (blob or arraybuffer).
    binary_type: BinaryType,
    /// Extensions negotiated.
    extensions: String,
    /// Message queue.
    message_queue: VecDeque<WebSocketMessage>,
    /// Event handlers.
    onopen: Option<JsValue>,
    onclose: Option<JsValue>,
    onerror: Option<JsValue>,
    onmessage: Option<JsValue>,
}

impl WebSocket {
    /// Create a new WebSocket.
    pub fn new(url: &str, protocols: Option<Vec<String>>) -> Self {
        Self {
            url: url.to_string(),
            ready_state: ReadyState::Connecting,
            buffered_amount: 0,
            protocol: protocols.and_then(|p| p.first().cloned()).unwrap_or_default(),
            binary_type: BinaryType::Blob,
            extensions: String::new(),
            message_queue: VecDeque::new(),
            onopen: None,
            onclose: None,
            onerror: None,
            onmessage: None,
        }
    }

    /// Get the URL.
    pub fn url(&self) -> &str {
        &self.url
    }

    /// Get the ready state.
    pub fn ready_state(&self) -> ReadyState {
        self.ready_state
    }

    /// Get the buffered amount.
    pub fn buffered_amount(&self) -> u64 {
        self.buffered_amount
    }

    /// Get the protocol.
    pub fn protocol(&self) -> &str {
        &self.protocol
    }

    /// Get the binary type.
    pub fn binary_type(&self) -> BinaryType {
        self.binary_type
    }

    /// Set the binary type.
    pub fn set_binary_type(&mut self, binary_type: BinaryType) {
        self.binary_type = binary_type;
    }

    /// Get extensions.
    pub fn extensions(&self) -> &str {
        &self.extensions
    }

    /// Send a text message.
    pub fn send_text(&mut self, data: &str) -> Result<(), WebSocketError> {
        if self.ready_state != ReadyState::Open {
            return Err(WebSocketError::InvalidState);
        }

        self.buffered_amount += data.len() as u64;
        self.message_queue.push_back(WebSocketMessage::Text(data.to_string()));
        Ok(())
    }

    /// Send binary data.
    pub fn send_binary(&mut self, data: Vec<u8>) -> Result<(), WebSocketError> {
        if self.ready_state != ReadyState::Open {
            return Err(WebSocketError::InvalidState);
        }

        self.buffered_amount += data.len() as u64;
        self.message_queue.push_back(WebSocketMessage::Binary(data));
        Ok(())
    }

    /// Close the connection.
    pub fn close(&mut self, code: Option<u16>, reason: Option<String>) -> Result<(), WebSocketError> {
        if self.ready_state == ReadyState::Closing || self.ready_state == ReadyState::Closed {
            return Ok(());
        }

        // Validate close code
        if let Some(code) = code {
            if code != 1000 && !(3000..=4999).contains(&code) {
                return Err(WebSocketError::InvalidCloseCode);
            }
        }

        // Validate reason length
        if let Some(ref reason) = reason {
            if reason.as_bytes().len() > 123 {
                return Err(WebSocketError::ReasonTooLong);
            }
        }

        self.ready_state = ReadyState::Closing;
        Ok(())
    }

    /// Simulate connection open (for testing).
    pub fn simulate_open(&mut self) {
        self.ready_state = ReadyState::Open;
    }

    /// Simulate connection close.
    pub fn simulate_close(&mut self, code: u16, reason: &str) {
        self.ready_state = ReadyState::Closed;
    }

    /// Simulate receiving a message.
    pub fn simulate_message(&mut self, message: WebSocketMessage) {
        self.message_queue.push_back(message);
    }

    /// Get next message from queue.
    pub fn next_message(&mut self) -> Option<WebSocketMessage> {
        self.message_queue.pop_front()
    }

    /// Register WebSocket class on the global object.
    pub fn register(context: &mut Context) {
        let websocket = ObjectInitializer::new(context)
            // Constants
            .property(js_string!("CONNECTING"), 0, Attribute::READONLY)
            .property(js_string!("OPEN"), 1, Attribute::READONLY)
            .property(js_string!("CLOSING"), 2, Attribute::READONLY)
            .property(js_string!("CLOSED"), 3, Attribute::READONLY)
            // Methods
            .function(NativeFunction::from_fn_ptr(websocket_send), js_string!("send"), 1)
            .function(NativeFunction::from_fn_ptr(websocket_close), js_string!("close"), 2)
            .build();

        context
            .register_global_property(js_string!("WebSocket"), websocket, Attribute::all())
            .expect("Failed to register WebSocket");
    }
}

/// Binary type for WebSocket.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinaryType {
    Blob,
    ArrayBuffer,
}

/// WebSocket message.
#[derive(Clone, Debug)]
pub enum WebSocketMessage {
    Text(String),
    Binary(Vec<u8>),
}

/// WebSocket error.
#[derive(Debug, Clone, thiserror::Error)]
pub enum WebSocketError {
    #[error("Invalid state")]
    InvalidState,
    #[error("Invalid close code")]
    InvalidCloseCode,
    #[error("Close reason too long")]
    ReasonTooLong,
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    #[error("Network error: {0}")]
    NetworkError(String),
}

/// WebSocket close event.
#[derive(Clone, Debug)]
pub struct CloseEvent {
    /// Close code.
    pub code: u16,
    /// Close reason.
    pub reason: String,
    /// Whether the connection was cleanly closed.
    pub was_clean: bool,
}

impl CloseEvent {
    pub fn new(code: u16, reason: String, was_clean: bool) -> Self {
        Self {
            code,
            reason,
            was_clean,
        }
    }
}

/// WebSocket message event.
#[derive(Clone, Debug)]
pub struct MessageEvent {
    /// Message data.
    pub data: WebSocketMessage,
    /// Origin.
    pub origin: String,
    /// Last event ID.
    pub last_event_id: String,
}

// Native function implementations
fn websocket_send(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let _data = args.get_or_undefined(0);
    Ok(JsValue::undefined())
}

fn websocket_close(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let _code = args.get(0);
    let _reason = args.get(1);
    Ok(JsValue::undefined())
}

/// Standard WebSocket close codes.
pub mod close_codes {
    pub const NORMAL_CLOSURE: u16 = 1000;
    pub const GOING_AWAY: u16 = 1001;
    pub const PROTOCOL_ERROR: u16 = 1002;
    pub const UNSUPPORTED_DATA: u16 = 1003;
    pub const NO_STATUS_RECEIVED: u16 = 1005;
    pub const ABNORMAL_CLOSURE: u16 = 1006;
    pub const INVALID_FRAME_PAYLOAD_DATA: u16 = 1007;
    pub const POLICY_VIOLATION: u16 = 1008;
    pub const MESSAGE_TOO_BIG: u16 = 1009;
    pub const MANDATORY_EXTENSION: u16 = 1010;
    pub const INTERNAL_ERROR: u16 = 1011;
    pub const SERVICE_RESTART: u16 = 1012;
    pub const TRY_AGAIN_LATER: u16 = 1013;
    pub const BAD_GATEWAY: u16 = 1014;
    pub const TLS_HANDSHAKE: u16 = 1015;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_websocket_creation() {
        let ws = WebSocket::new("wss://example.com/socket", None);
        assert_eq!(ws.url(), "wss://example.com/socket");
        assert_eq!(ws.ready_state(), ReadyState::Connecting);
    }

    #[test]
    fn test_websocket_send() {
        let mut ws = WebSocket::new("wss://example.com/socket", None);

        // Can't send while connecting
        assert!(ws.send_text("hello").is_err());

        // Simulate connection
        ws.simulate_open();
        assert!(ws.send_text("hello").is_ok());
        assert_eq!(ws.buffered_amount(), 5);
    }

    #[test]
    fn test_websocket_close() {
        let mut ws = WebSocket::new("wss://example.com/socket", None);
        ws.simulate_open();

        // Valid close
        assert!(ws.close(Some(1000), Some("Normal closure".to_string())).is_ok());
        assert_eq!(ws.ready_state(), ReadyState::Closing);

        // Invalid close code
        let mut ws2 = WebSocket::new("wss://example.com/socket", None);
        ws2.simulate_open();
        assert!(ws2.close(Some(999), None).is_err());
    }

    #[test]
    fn test_websocket_messages() {
        let mut ws = WebSocket::new("wss://example.com/socket", None);
        ws.simulate_open();

        ws.simulate_message(WebSocketMessage::Text("hello".to_string()));
        ws.simulate_message(WebSocketMessage::Binary(vec![1, 2, 3]));

        match ws.next_message() {
            Some(WebSocketMessage::Text(s)) => assert_eq!(s, "hello"),
            _ => panic!("Expected text message"),
        }

        match ws.next_message() {
            Some(WebSocketMessage::Binary(b)) => assert_eq!(b, vec![1, 2, 3]),
            _ => panic!("Expected binary message"),
        }

        assert!(ws.next_message().is_none());
    }
}
