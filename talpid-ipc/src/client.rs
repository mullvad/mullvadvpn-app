use std::sync::{mpsc, Arc, Mutex, MutexGuard};
use std::thread;

use error_chain::ChainedError;
use serde;
use serde_json::{self, Result as JsonResult, Value as JsonValue};
use url;
use ws;

type JsonMap = serde_json::map::Map<String, JsonValue>;

mod errors {
    error_chain! {
        errors {
            ErrorResponse(error_message: String) {
                description("Received an RPC error response")
                display("Received an RPC error response: {}", error_message)
            }

            InvalidJsonRpcResponse(details: &'static str) {
                description("Received an invalid JSON-RPC response")
                display("Received an invalid JSON-RPC response: {}", details)
            }

            WebSocketError {
                description("Error with WebSocket connection")
            }
        }
    }
}
pub use self::errors::*;


struct Factory {
    active_request: Arc<Mutex<Option<(i64, mpsc::Sender<Result<JsonValue>>)>>>,
    sender_tx: mpsc::Sender<ws::Sender>,
}

impl ws::Factory for Factory {
    type Handler = Handler;

    fn connection_made(&mut self, sender: ws::Sender) -> Self::Handler {
        trace!("Connection established");

        let _ = self.sender_tx.send(sender);

        Handler {
            active_request: self.active_request.clone(),
        }
    }
}


struct Handler {
    active_request: Arc<Mutex<Option<(i64, mpsc::Sender<Result<JsonValue>>)>>>,
}

impl Handler {
    fn process_message(&mut self, msg: ws::Message) -> Result<()> {
        trace!("WsIpcClient incoming message: {:?}", msg);
        let mut response_json_object = self.parse_message_object(msg)?;
        let response_id = self.parse_response_id(&mut response_json_object)?;
        let rpc_result = self.parse_response_result(response_json_object);

        let mut active_request = self.lock_active_request();

        if let Some((request_id, request_result)) = active_request.take() {
            if response_id == request_id {
                let _ = request_result.send(rpc_result);
            } else {
                warn!("Received an unexpect JSON-RPC message");
                *active_request = Some((request_id, request_result));
            }
        }

        Ok(())
    }

    fn parse_message_object(&self, msg: ws::Message) -> Result<JsonMap> {
        let parsed_json: JsonResult<JsonValue> = match msg {
            ws::Message::Text(s) => serde_json::from_str(&s),
            ws::Message::Binary(b) => serde_json::from_slice(&b),
        };
        let json = parsed_json.chain_err(|| {
            ErrorKind::InvalidJsonRpcResponse("Unable to deserialize ws message as JSON")
        })?;

        let mut json_object_map = match json {
            JsonValue::Object(object_map) => object_map,
            _ => bail!(ErrorKind::InvalidJsonRpcResponse(
                "Received response is not a JSON object"
            )),
        };

        ensure!(
            json_object_map.remove("jsonrpc") == Some(JsonValue::String("2.0".to_owned())),
            ErrorKind::InvalidJsonRpcResponse("Invalid JSON-RPC version field in response")
        );

        Ok(json_object_map)
    }

    fn parse_response_id(&self, json_object_map: &mut JsonMap) -> Result<i64> {
        match json_object_map.remove("id") {
            Some(JsonValue::Number(id)) => id.as_i64().ok_or_else(|| {
                ErrorKind::InvalidJsonRpcResponse("Invalid request ID number").into()
            }),
            None => Err(ErrorKind::InvalidJsonRpcResponse("Missing request ID").into()),
            _ => Err(ErrorKind::InvalidJsonRpcResponse("Invalid request ID value").into()),
        }
    }

    fn parse_response_result(&self, mut json_object_map: JsonMap) -> Result<JsonValue> {
        let result = json_object_map.remove("result");
        let error = json_object_map.remove("error");

        match (result, error) {
            (Some(remote_result), None) => Ok(remote_result),
            (None, Some(JsonValue::String(remote_error))) => {
                Err(ErrorKind::ErrorResponse(remote_error).into())
            }
            (None, Some(json_value)) => {
                Err(ErrorKind::ErrorResponse(json_value.to_string()).into())
            }
            (None, None) => Err(ErrorKind::InvalidJsonRpcResponse("Missing RPC result").into()),
            (Some(_), Some(_)) => Err(ErrorKind::InvalidJsonRpcResponse(
                "Response is ambiguous, contains both a successful result and an error",
            ).into()),
        }
    }

    fn lock_active_request(
        &mut self,
    ) -> MutexGuard<Option<(i64, mpsc::Sender<Result<JsonValue>>)>> {
        self.active_request
            .lock()
            .expect("a thread panicked while using the active JSON-RPC request")
    }
}

impl ws::Handler for Handler {
    fn on_message(&mut self, msg: ws::Message) -> ws::Result<()> {
        if let Err(error) = self.process_message(msg) {
            let chained_error = error.chain_err(|| "Failed to process RPC message");
            error!("{}", chained_error.display_chain());
        }

        Ok(())
    }

    fn on_error(&mut self, error: ws::Error) {
        if let Some((_id, ref mut request_result)) = *self.lock_active_request() {
            let _ = request_result.send(Err(error).chain_err(|| ErrorKind::WebSocketError));
        }
    }
}


pub struct WsIpcClient {
    url: url::Url,
    next_id: i64,
    active_request: Arc<Mutex<Option<(i64, mpsc::Sender<Result<JsonValue>>)>>>,
    sender: Option<Mutex<ws::Sender>>,
}

impl WsIpcClient {
    pub fn new(server_id: ::IpcServerId) -> Result<Self> {
        let url = url::Url::parse(&server_id).chain_err(|| "Unable to parse server_id as url")?;
        let active_request = Arc::new(Mutex::new(None));

        Ok(WsIpcClient {
            url,
            next_id: 1,
            active_request,
            sender: None,
        })
    }

    pub fn connect(&mut self) -> Result<()> {
        let sender_rx = self.open_websocket()?;
        let sender = sender_rx
            .recv()
            .chain_err(|| "WebSocket connection failed")?;

        self.sender = Some(Mutex::new(sender));

        Ok(())
    }

    fn open_websocket(&mut self) -> Result<mpsc::Receiver<ws::Sender>> {
        let (sender_tx, sender_rx) = mpsc::channel();
        let factory = Factory {
            active_request: self.active_request.clone(),
            sender_tx,
        };

        let mut websocket = ws::WebSocket::new(factory).chain_err(|| "Unable to create WebSocket")?;

        websocket
            .connect(self.url.clone())
            .chain_err(|| "Unable to connect WebSocket to URL")?;

        thread::spawn(move || {
            let result = websocket
                .run()
                .chain_err(|| "Error while running WebSocket event loop");

            if let Err(error) = result {
                error!("{}", error.display_chain());
            }
        });

        Ok(sender_rx)
    }

    pub fn call<T, O>(&mut self, method: &str, params: &T) -> Result<O>
    where
        T: serde::Serialize,
        O: for<'de> serde::Deserialize<'de>,
    {
        if self.sender.is_none() {
            self.connect()?;
        }

        let id = self.new_id();
        let (result_tx, result_rx) = mpsc::channel();

        self.queue_request_response(id, result_tx);
        self.send_request(id, method, params)?;

        let json_result = result_rx.recv().chain_err(|| "No response received")?;

        Ok(serde_json::from_value(json_result?).chain_err(|| "Failed to deserialize RPC result")?)
    }

    fn new_id(&mut self) -> i64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    fn queue_request_response(&mut self, id: i64, result_tx: mpsc::Sender<Result<JsonValue>>) {
        let mut active_request = self.active_request
            .lock()
            .expect("a thread panicked using the active RPC request map");

        *active_request = Some((id, result_tx));
    }

    fn send_request<T>(&mut self, id: i64, method: &str, params: &T) -> Result<()>
    where
        T: serde::Serialize,
    {
        let json_request = self.build_json_request(id, method, params);

        self.sender
            .as_ref()
            .expect("attempt to send without an established connection")
            .lock()
            .expect("a thread panicked while using the WebSocket sender")
            .send(json_request.as_bytes())
            .chain_err(|| "Unable to send jsonrpc request")
    }

    fn build_json_request<T>(&mut self, id: i64, method: &str, params: &T) -> String
    where
        T: serde::Serialize,
    {
        let request_json = json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params,
        });
        format!("{}", request_json)
    }
}
