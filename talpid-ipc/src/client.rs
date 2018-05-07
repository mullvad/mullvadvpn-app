use std::collections::HashMap;
use std::sync::{mpsc, Arc, Mutex, MutexGuard, Weak};
use std::thread;

use error_chain::ChainedError;
use jsonrpc_pubsub::SubscriptionId;
use serde;
use serde_json::{self, Result as JsonResult, Value as JsonValue};
use url::Url;
use ws;

type JsonMap = serde_json::map::Map<String, JsonValue>;

mod errors {
    error_chain! {
        errors {
            ConnectError(details: &'static str) {
                description("Failed to connect to RPC server")
                display("Failed to connect to RPC server: {}", details)
            }

            ErrorResponse(error_message: String) {
                description("Received an RPC error response")
                display("Received an RPC error response: {}", error_message)
            }

            DeserializeResponseError {
                description("Failed to deserialize response")
            }

            DeserializeSubscriptionEvent(event: String) {
                description("Failed to deserialize RPC subscription event")
                display("Failed to deserialize RPC subscription event {}", event)
            }

            ForwardSubscriptionEvent(event: String) {
                description("Failed to forward RPC subscription event")
                display("Failed to forward RPC subscription event {}", event)
            }

            InvalidJsonRpcResponse(details: &'static str) {
                description("Received an invalid JSON-RPC response")
                display("Received an invalid JSON-RPC response: {}", details)
            }

            InvalidSubscriptionEvent(details: &'static str) {
                description("Received an invalid JSON-RPC PubSub event")
                display("Received an invalid JSON-RPC PubSub event: {}", details)
            }

            InvalidSubscriptionId(raw_id: ::serde_json::Value) {
                description("Received an invalid JSON-RPC subscription ID for subscribe request")
                display(
                    "Received an invalid JSON-RPC subscription ID for subscribe request: {}",
                    raw_id,
                )
            }

            InvalidServerIdUrl(server_id: ::IpcServerId) {
                description("Unable to parse given server ID as a URL")
                display("Unable to parse given server ID as a URL: {}", server_id)
            }

            MissingResponse {
                description("No response received")
            }

            SendRequestError {
                description("Failed to send JSON-RPC request")
            }

            WebSocketError {
                description("Error with WebSocket connection")
            }
        }
    }
}
pub use self::errors::*;

#[derive(Debug, Eq, PartialEq)]
pub enum SubscriptionHandlerResult {
    Active,
    Finished,
}

type SubscriptionHandler = Box<Fn(JsonValue) -> SubscriptionHandlerResult + Send>;

struct ActiveRequest {
    id: i64,
    response_tx: mpsc::Sender<Result<JsonValue>>,
}

impl ActiveRequest {
    pub fn new(id: i64, response_tx: mpsc::Sender<Result<JsonValue>>) -> Self {
        ActiveRequest { id, response_tx }
    }

    pub fn id(&self) -> i64 {
        self.id
    }

    pub fn send_response(&mut self, response: Result<JsonValue>) {
        let _ = self.response_tx.send(response);
    }
}

struct Factory {
    active_request: Arc<Mutex<Option<ActiveRequest>>>,
    active_subscriptions: Arc<Mutex<HashMap<SubscriptionId, SubscriptionHandler>>>,
    sender_tx: mpsc::Sender<ws::Sender>,
}

impl ws::Factory for Factory {
    type Handler = Handler;

    fn connection_made(&mut self, sender: ws::Sender) -> Self::Handler {
        trace!("Connection established");

        let _ = self.sender_tx.send(sender);

        Handler {
            active_request: self.active_request.clone(),
            active_subscriptions: self.active_subscriptions.clone(),
        }
    }
}


struct Handler {
    active_request: Arc<Mutex<Option<ActiveRequest>>>,
    active_subscriptions: Arc<Mutex<HashMap<SubscriptionId, SubscriptionHandler>>>,
}

impl Handler {
    fn process_message(&mut self, msg: ws::Message) -> Result<()> {
        trace!("WsIpcClient incoming message: {:?}", msg);
        let mut message_json_object = self.parse_message_object(msg)?;
        let response_id = self.parse_response_id(&mut message_json_object)?;

        if let Some(id) = response_id {
            self.process_response(id, message_json_object)
        } else {
            self.process_notification(message_json_object)
        }
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

    fn parse_response_id(&self, json_object_map: &mut JsonMap) -> Result<Option<i64>> {
        match json_object_map.remove("id") {
            Some(JsonValue::Number(id)) => id.as_i64().map(Some).ok_or_else(|| {
                ErrorKind::InvalidJsonRpcResponse("Invalid request ID number").into()
            }),
            Some(_) => Err(ErrorKind::InvalidJsonRpcResponse("Invalid request ID value").into()),
            None => Ok(None),
        }
    }

    fn process_response(&mut self, response_id: i64, response: JsonMap) -> Result<()> {
        let rpc_result = self.parse_response_result(response);
        let mut active_request = self.lock_active_request();

        if let Some(mut request) = active_request.take() {
            if response_id == request.id() {
                let _ = request.send_response(rpc_result);
            } else {
                warn!("Received an unexpect JSON-RPC message");
                *active_request = Some(request);
            }
        }

        Ok(())
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

    fn process_notification(&mut self, notification: JsonMap) -> Result<()> {
        let (subscription_id, event) = self.parse_subscription_event(notification)?;
        let mut active_subscriptions = self.active_subscriptions
            .lock()
            .expect("a thread panicked while using the active subscriptions map");

        let should_remove_handler =
            if let Some(handle_subscription) = active_subscriptions.get(&subscription_id) {
                handle_subscription(event) == SubscriptionHandlerResult::Finished
            } else {
                warn!("Received an unexpected notification");
                false
            };

        if should_remove_handler {
            active_subscriptions.remove(&subscription_id);
        }

        Ok(())
    }

    fn parse_subscription_event(
        &mut self,
        mut notification: JsonMap,
    ) -> Result<(SubscriptionId, JsonValue)> {
        match notification.remove("params") {
            Some(JsonValue::Object(mut parameters)) => {
                let raw_id = parameters
                    .remove("subscription")
                    .ok_or_else(|| ErrorKind::InvalidSubscriptionEvent("Missing subscription ID"))?;
                let id = SubscriptionId::parse_value(&raw_id)
                    .ok_or_else(|| ErrorKind::InvalidSubscriptionEvent("Invalid subscription ID"))?;
                let event = parameters
                    .remove("result")
                    .ok_or_else(|| ErrorKind::InvalidSubscriptionEvent("Missing event data"))?;

                Ok((id, event))
            }
            Some(_) => bail!(ErrorKind::InvalidSubscriptionEvent(
                "RPC parameters is not a JSON object map"
            )),
            None => bail!(ErrorKind::InvalidSubscriptionEvent(
                "Missing RPC parameters"
            )),
        }
    }

    fn lock_active_request(&mut self) -> MutexGuard<Option<ActiveRequest>> {
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
        if let Some(active_request) = self.lock_active_request().as_mut() {
            active_request.send_response(Err(error).chain_err(|| ErrorKind::WebSocketError));
        }
    }
}


pub struct WsIpcClient {
    next_id: Arc<Mutex<i64>>,
    active_request: Arc<Mutex<Option<ActiveRequest>>>,
    active_subscriptions: Arc<Mutex<HashMap<SubscriptionId, SubscriptionHandler>>>,
    sender: Arc<Mutex<ws::Sender>>,
}

impl WsIpcClient {
    pub fn connect(server_id: &::IpcServerId) -> Result<Self> {
        let url = Url::parse(&server_id)
            .chain_err(|| ErrorKind::InvalidServerIdUrl(server_id.to_owned()))?;
        let active_request = Arc::new(Mutex::new(None));
        let active_subscriptions = Arc::new(Mutex::new(HashMap::new()));
        let sender =
            Self::open_websocket(url, active_request.clone(), active_subscriptions.clone())?;

        Ok(WsIpcClient {
            next_id: Arc::new(Mutex::new(1)),
            active_request,
            active_subscriptions,
            sender,
        })
    }

    fn open_websocket(
        url: Url,
        active_request: Arc<Mutex<Option<ActiveRequest>>>,
        active_subscriptions: Arc<Mutex<HashMap<SubscriptionId, SubscriptionHandler>>>,
    ) -> Result<Arc<Mutex<ws::Sender>>> {
        let (sender_tx, sender_rx) = mpsc::channel();
        let factory = Factory {
            active_request,
            active_subscriptions,
            sender_tx,
        };

        let mut websocket = ws::WebSocket::new(factory)
            .chain_err(|| ErrorKind::ConnectError("Unable to create WebSocket"))?;

        websocket
            .connect(url)
            .chain_err(|| ErrorKind::ConnectError("Unable to connect WebSocket to URL"))?;

        thread::spawn(move || {
            let result = websocket
                .run()
                .chain_err(|| ErrorKind::ConnectError("Error while running WebSocket event loop"));

            if let Err(error) = result {
                error!("{}", error.display_chain());
            }
        });

        sender_rx
            .recv()
            .map(|sender| Arc::new(Mutex::new(sender)))
            .chain_err(|| ErrorKind::ConnectError("WebSocket connection failed"))
    }

    pub fn subscribe<V, M>(&self, event: &str, sender: mpsc::Sender<M>) -> Result<()>
    where
        V: for<'de> serde::Deserialize<'de>,
        M: From<V> + Send + 'static,
    {
        let raw_subscription_id = self.call(&format!("{}_subscribe", event), &[] as &[u8; 0])?;
        let subscription_id = SubscriptionId::parse_value(&raw_subscription_id)
            .ok_or_else(|| ErrorKind::InvalidSubscriptionId(raw_subscription_id))?;

        let event_name = event.to_owned();
        let unsubscriber = self.downgrade();
        let unsubscribe_id = subscription_id.clone();
        let handler =
            move |json_value| match forward_subscription_event(&event_name, json_value, &sender) {
                Ok(()) => SubscriptionHandlerResult::Active,
                Err(error) => {
                    error!("{}", error.display_chain());
                    unsubscriber.unsubscribe(&event_name, &unsubscribe_id);
                    SubscriptionHandlerResult::Finished
                }
            };

        self.register_subscription(subscription_id, handler);

        Ok(())
    }

    fn downgrade(&self) -> WeakWsIpcClient {
        WeakWsIpcClient {
            next_id: Arc::downgrade(&self.next_id),
            active_request: Arc::downgrade(&self.active_request),
            active_subscriptions: Arc::downgrade(&self.active_subscriptions),
            sender: Arc::downgrade(&self.sender),
        }
    }

    fn register_subscription<H>(&self, id: SubscriptionId, handler: H)
    where
        H: Fn(JsonValue) -> SubscriptionHandlerResult + Send + 'static,
    {
        self.lock_active_subscriptions()
            .insert(id, Box::new(handler));
    }

    fn lock_active_subscriptions(
        &self,
    ) -> MutexGuard<HashMap<SubscriptionId, SubscriptionHandler>> {
        self.active_subscriptions
            .lock()
            .expect("a thread panicked while using the active subscriptions map")
    }

    pub fn unsubscribe(&self, event: &str, id: SubscriptionId) -> Result<()> {
        let method = format!("{}_unsubscribe", event);

        self.lock_active_subscriptions().remove(&id);

        match id {
            SubscriptionId::Number(raw_id) => self.call(&method, &[raw_id]),
            SubscriptionId::String(raw_id) => self.call(&method, &[raw_id]),
        }
    }

    pub fn call<T, O>(&self, method: &str, params: &T) -> Result<O>
    where
        T: serde::Serialize,
        O: for<'de> serde::Deserialize<'de>,
    {
        let id = self.new_id();
        let (result_tx, result_rx) = mpsc::channel();

        self.queue_request_response(id, result_tx);
        self.send_request(id, method, params)?;

        let json_result = result_rx.recv().chain_err(|| ErrorKind::MissingResponse)?;

        Ok(serde_json::from_value(json_result?).chain_err(|| ErrorKind::DeserializeResponseError)?)
    }

    fn new_id(&self) -> i64 {
        let mut next_id = self.next_id
            .lock()
            .expect("a thread panicked generating a JSON-RPC request ID");

        let id = *next_id;
        *next_id += 1;
        id
    }

    fn queue_request_response(&self, id: i64, result_tx: mpsc::Sender<Result<JsonValue>>) {
        let mut active_request = self.active_request
            .lock()
            .expect("a thread panicked using the active RPC request map");

        *active_request = Some(ActiveRequest::new(id, result_tx));
    }

    fn send_request<T>(&self, id: i64, method: &str, params: &T) -> Result<()>
    where
        T: serde::Serialize,
    {
        let json_request = self.build_json_request(id, method, params);

        self.sender
            .lock()
            .expect("a thread panicked while sending a JSON-RPC request")
            .send(json_request.as_bytes())
            .chain_err(|| ErrorKind::SendRequestError)
    }

    fn build_json_request<T>(&self, id: i64, method: &str, params: &T) -> String
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

fn forward_subscription_event<V, M>(
    event_name: &String,
    json_value: JsonValue,
    sender: &mpsc::Sender<M>,
) -> Result<()>
where
    V: for<'de> serde::Deserialize<'de>,
    M: From<V> + Send + 'static,
{
    let value: V = serde_json::from_value(json_value)
        .chain_err(|| ErrorKind::DeserializeSubscriptionEvent(event_name.clone()))?;
    let message = value.into();

    sender
        .send(message)
        .chain_err(|| ErrorKind::ForwardSubscriptionEvent(event_name.clone()))
}

struct WeakWsIpcClient {
    next_id: Weak<Mutex<i64>>,
    active_request: Weak<Mutex<Option<ActiveRequest>>>,
    active_subscriptions: Weak<Mutex<HashMap<SubscriptionId, SubscriptionHandler>>>,
    sender: Weak<Mutex<ws::Sender>>,
}

impl WeakWsIpcClient {
    pub fn upgrade(&self) -> Option<WsIpcClient> {
        let next_id = self.next_id.upgrade()?;
        let active_request = self.active_request.upgrade()?;
        let active_subscriptions = self.active_subscriptions.upgrade()?;
        let sender = self.sender.upgrade()?;

        Some(WsIpcClient {
            next_id,
            active_request,
            active_subscriptions,
            sender,
        })
    }

    pub fn unsubscribe(&self, event: &str, id: &SubscriptionId) {
        if let Some(upgraded_self) = self.upgrade() {
            if let Err(error) = upgraded_self.unsubscribe(event, id.clone()) {
                let chained_error = Error::with_chain(
                    error,
                    format!("Failed to unsubscribe from events of type {}", event),
                );
                error!("{}", chained_error.display_chain());
            }
        }
    }
}
