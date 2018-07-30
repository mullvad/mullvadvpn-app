use std::collections::HashMap;
use std::sync::mpsc;
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

            ConnectionHandlerStopped {
                description("The WebSocket connection handler thread has stopped")
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

            InvalidServerIdUrl(server_id: ::IpcServerId) {
                description("Unable to parse given server ID as a URL")
                display("Unable to parse given server ID as a URL: {}", server_id)
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

            MissingResponse {
                description("No response received")
            }

            SendRequestError(method: String) {
                description("Failed to send a request to call a remote JSON-RPC procedure")
                display(
                    "Failed to send a request to call the \"{}\" remote JSON-RPC procedure",
                    method
                )
            }

            SerializeArgumentsError {
                description("Failed to serialize JSON-RPC request arguments")
            }

            SerializeSubscriptionId {
                description("Failed to serialize JSON-RPC subscription ID")
            }

            UnsubscribeError {
                description("Failed to unsubscribe from a remote event")
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

enum WsIpcCommand {
    Call {
        method: String,
        arguments: JsonValue,
        response_tx: mpsc::Sender<Result<JsonValue>>,
    },

    Subscribe {
        id: SubscriptionId,
        handler: SubscriptionHandler,
        unsubscribe_method: String,
    },

    Response {
        id: i64,
        result: Result<JsonValue>,
    },

    Notification {
        subscription: SubscriptionId,
        event: JsonValue,
    },

    Error(Error),
}

struct Factory {
    connection_tx: mpsc::Sender<WsIpcCommand>,
    sender_tx: mpsc::Sender<ws::Sender>,
}

impl ws::Factory for Factory {
    type Handler = Handler;

    fn connection_made(&mut self, sender: ws::Sender) -> Self::Handler {
        trace!("Connection established");

        let _ = self.sender_tx.send(sender);

        Handler {
            connection_tx: self.connection_tx.clone(),
        }
    }
}


struct Handler {
    connection_tx: mpsc::Sender<WsIpcCommand>,
}

impl Handler {
    fn process_message(&mut self, msg: ws::Message) -> Result<()> {
        trace!("WsIpcClient incoming message: {:?}", msg);
        let mut message_json_object = self.parse_message_object(msg)?;
        let response_id = self.parse_response_id(&mut message_json_object)?;

        let command = if let Some(id) = response_id {
            let result = self.parse_response_result(message_json_object);

            WsIpcCommand::Response { id, result }
        } else {
            let (subscription, event) = self.parse_subscription_event(message_json_object)?;

            WsIpcCommand::Notification {
                subscription,
                event,
            }
        };

        self.connection_tx
            .send(command)
            .chain_err(|| ErrorKind::ConnectionHandlerStopped)
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

    fn parse_subscription_event(
        &mut self,
        mut notification: JsonMap,
    ) -> Result<(SubscriptionId, JsonValue)> {
        match notification.remove("params") {
            Some(JsonValue::Object(mut parameters)) => {
                let raw_id = parameters.remove("subscription").ok_or_else(|| {
                    ErrorKind::InvalidSubscriptionEvent("Missing subscription ID")
                })?;
                let id = SubscriptionId::parse_value(&raw_id).ok_or_else(|| {
                    ErrorKind::InvalidSubscriptionEvent("Invalid subscription ID")
                })?;
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
        let error = Error::with_chain(error, ErrorKind::WebSocketError);

        let _ = self.connection_tx.send(WsIpcCommand::Error(error));
    }
}

pub struct WsIpcClient {
    connection_tx: mpsc::Sender<WsIpcCommand>,
}

impl WsIpcClient {
    pub fn connect(server_id: &::IpcServerId) -> Result<Self> {
        let url = Url::parse(&server_id)
            .chain_err(|| ErrorKind::InvalidServerIdUrl(server_id.to_owned()))?;
        let (connection_tx, connection_rx) = mpsc::channel();
        let sender = Self::open_websocket(url, connection_tx.clone())?;

        WsIpcClientConnection::spawn(sender, connection_rx);

        Ok(WsIpcClient { connection_tx })
    }

    fn open_websocket(url: Url, connection_tx: mpsc::Sender<WsIpcCommand>) -> Result<ws::Sender> {
        let (sender_tx, sender_rx) = mpsc::channel();
        let factory = Factory {
            connection_tx,
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
            .chain_err(|| ErrorKind::ConnectError("WebSocket connection failed"))
    }

    pub fn subscribe<V, M>(
        &mut self,
        subscribe_method: String,
        unsubscribe_method: String,
        sender: mpsc::Sender<M>,
    ) -> Result<()>
    where
        V: for<'de> serde::Deserialize<'de>,
        M: From<V> + Send + 'static,
    {
        let raw_subscription_id = self.call(&subscribe_method, &[] as &[u8; 0])?;
        let subscription_id = SubscriptionId::parse_value(&raw_subscription_id)
            .ok_or_else(|| ErrorKind::InvalidSubscriptionId(raw_subscription_id))?;

        let handler = move |json_value| match forward_subscription_event(
            &subscribe_method,
            json_value,
            &sender,
        ) {
            Ok(()) => SubscriptionHandlerResult::Active,
            Err(error) => {
                error!("{}", error.display_chain());
                SubscriptionHandlerResult::Finished
            }
        };

        self.register_subscription(subscription_id, handler, unsubscribe_method)?;

        Ok(())
    }

    fn register_subscription<H>(
        &mut self,
        id: SubscriptionId,
        handler: H,
        unsubscribe_method: String,
    ) -> Result<()>
    where
        H: Fn(JsonValue) -> SubscriptionHandlerResult + Send + 'static,
    {
        self.connection_tx
            .send(WsIpcCommand::Subscribe {
                id,
                handler: Box::new(handler),
                unsubscribe_method,
            }).chain_err(|| ErrorKind::ConnectionHandlerStopped)
    }

    pub fn call<S, T, O>(&mut self, method: S, params: &T) -> Result<O>
    where
        S: ToString,
        T: serde::Serialize,
        O: for<'de> serde::Deserialize<'de>,
    {
        let arguments =
            serde_json::to_value(params).chain_err(|| ErrorKind::SerializeArgumentsError)?;
        let (response_tx, response_rx) = mpsc::channel();
        let command = WsIpcCommand::Call {
            method: method.to_string(),
            arguments,
            response_tx,
        };

        self.connection_tx
            .send(command)
            .chain_err(|| ErrorKind::ConnectionHandlerStopped)?;

        let json_result = response_rx
            .recv()
            .chain_err(|| ErrorKind::MissingResponse)?;

        Ok(serde_json::from_value(json_result?)
            .chain_err(|| ErrorKind::DeserializeResponseError)?)
    }
}

struct WsIpcClientConnection {
    next_id: i64,
    active_request: Option<ActiveRequest>,
    active_subscriptions: HashMap<SubscriptionId, (SubscriptionHandler, String)>,
    sender: ws::Sender,
}

impl WsIpcClientConnection {
    pub fn spawn(sender: ws::Sender, commands: mpsc::Receiver<WsIpcCommand>) {
        let mut instance = WsIpcClientConnection {
            next_id: 1,
            active_request: None,
            active_subscriptions: HashMap::new(),
            sender,
        };

        thread::spawn(move || {
            if let Err(error) = instance.run(commands) {
                let chained_error = Error::with_chain(error, "WsIpcClient event loop error");
                error!("{}", chained_error.display_chain());
            }
        });
    }

    fn run(&mut self, commands: mpsc::Receiver<WsIpcCommand>) -> Result<()> {
        use self::WsIpcCommand::*;

        for command in commands {
            match command {
                Call {
                    method,
                    arguments,
                    response_tx,
                } => self.call(method, arguments, response_tx)?,
                Subscribe {
                    id,
                    handler,
                    unsubscribe_method,
                } => {
                    self.active_subscriptions
                        .insert(id, (handler, unsubscribe_method));
                }
                Response { id, result } => self.handle_response(id, result)?,
                Notification {
                    subscription,
                    event,
                } => self.handle_notification(subscription, event)?,
                Error(error) => self.handle_error(error),
            }
        }

        Ok(())
    }

    fn call(
        &mut self,
        method: String,
        arguments: JsonValue,
        response_tx: mpsc::Sender<Result<JsonValue>>,
    ) -> Result<()> {
        let id = self.new_id();
        self.queue_request_response(id, response_tx);
        self.send_request(id, method, arguments)
    }

    fn new_id(&mut self) -> i64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    fn queue_request_response(&mut self, id: i64, response_tx: mpsc::Sender<Result<JsonValue>>) {
        self.active_request = Some(ActiveRequest::new(id, response_tx));
    }

    fn send_request(&mut self, id: i64, method: String, arguments: JsonValue) -> Result<()> {
        let json_request = self.build_json_request(id, &method, arguments);

        self.sender
            .send(json_request.as_bytes())
            .chain_err(|| ErrorKind::SendRequestError(method))
    }

    fn build_json_request(&mut self, id: i64, method: &str, params: JsonValue) -> String {
        let request_json = json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params,
        });
        format!("{}", request_json)
    }

    fn handle_response(&mut self, id: i64, result: Result<JsonValue>) -> Result<()> {
        if let Some(mut request) = self.active_request.take() {
            if request.id() == id {
                request.send_response(result);
            } else {
                self.active_request = Some(request);
                warn!("Received an unexpected response with ID {}", id);
            }
        } else {
            warn!("Received an unexpected response with ID {}", id);
        }

        Ok(())
    }

    fn handle_notification(&mut self, id: SubscriptionId, event: JsonValue) -> Result<()> {
        let unsubscribe_method =
            if let Some((handler, unsubscribe_method)) = self.active_subscriptions.get(&id) {
                match handler(event) {
                    SubscriptionHandlerResult::Active => None,
                    SubscriptionHandlerResult::Finished => Some(unsubscribe_method.clone()),
                }
            } else {
                warn!("Received an unexpected notification");
                None
            };

        if let Some(method) = unsubscribe_method {
            self.unsubscribe(method, id)?;
        }

        Ok(())
    }

    fn unsubscribe(&mut self, method: String, id: SubscriptionId) -> Result<()> {
        self.active_subscriptions.remove(&id);

        let (result_tx, _) = mpsc::channel();
        let arguments = match id {
            SubscriptionId::Number(id) => serde_json::to_value(&[id]),
            SubscriptionId::String(id) => serde_json::to_value(&[id]),
        }.chain_err(|| ErrorKind::SerializeSubscriptionId);

        self.call(method, arguments?, result_tx)
            .chain_err(|| ErrorKind::UnsubscribeError)
    }

    fn handle_error(&mut self, error: Error) {
        if let Some(ref mut request) = self.active_request {
            let _ = request.response_tx.send(Err(error));
        } else {
            error!("{}", error.display_chain());
        }
    }
}

fn forward_subscription_event<V, M>(
    subscribe_method: &String,
    json_value: JsonValue,
    sender: &mpsc::Sender<M>,
) -> Result<()>
where
    V: for<'de> serde::Deserialize<'de>,
    M: From<V> + Send + 'static,
{
    let value: V = serde_json::from_value(json_value)
        .chain_err(|| ErrorKind::DeserializeSubscriptionEvent(subscribe_method.clone()))?;
    let message = M::from(value);

    sender
        .send(message)
        .chain_err(|| ErrorKind::ForwardSubscriptionEvent(subscribe_method.clone()))
}
