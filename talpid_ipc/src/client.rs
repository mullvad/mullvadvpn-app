use serde;
use serde_json;
use std::sync::mpsc;
use url;
use ws;

mod errors {
    error_chain!{}
}
pub use self::errors::*;


struct Factory<O: for<'de> serde::Deserialize<'de>> {
    request: String,
    result_tx: mpsc::Sender<Result<O>>,
}

impl<O: for<'de> serde::Deserialize<'de>> ws::Factory for Factory<O> {
    type Handler = Handler<O>;

    fn connection_made(&mut self, sender: ws::Sender) -> Self::Handler {
        debug!("Sending: {}", self.request);
        if let Err(e) =
            sender.send(&self.request[..]).chain_err(|| "Unable to send jsonrpc request") {
            self.result_tx.send(Err(e)).unwrap();
        }
        Handler {
            sender,
            result_tx: self.result_tx.clone(),
        }
    }
}


struct Handler<O: for<'de> serde::Deserialize<'de>> {
    sender: ws::Sender,
    result_tx: mpsc::Sender<Result<O>>,
}

impl<O: for<'de> serde::Deserialize<'de>> Handler<O> {
    fn parse_reply(&self, msg: ws::Message) -> Result<O> {
        let json: serde_json::Value =
            match msg {
                    ws::Message::Text(s) => serde_json::from_str(&s),
                    ws::Message::Binary(b) => serde_json::from_slice(&b),
                }
                .chain_err(|| "Unable to deserialize ws message as JSON")?;
        let result: Option<serde_json::Value> = match json {
            serde_json::Value::Object(mut map) => map.remove("result"),
            _ => None,
        };
        match result {
            Some(result) => {
                serde_json::from_value(result)
                    .chain_err(|| "Unable to deserialize result into derisred type")
            }
            None => bail!("Invalid reply, no 'result' field"),
        }
    }
}

impl<O: for<'de> serde::Deserialize<'de>> ws::Handler for Handler<O> {
    fn on_message(&mut self, msg: ws::Message) -> ws::Result<()> {
        debug!("WsIpcClient incoming message: {:?}", msg);
        let reply_result = self.parse_reply(msg);
        let close_result = self.sender.close(ws::CloseCode::Normal);
        if let Err(e) = close_result.chain_err(|| "Unable to close WebSocket") {
            self.result_tx.send(Err(e)).unwrap();
        }
        self.result_tx.send(reply_result).unwrap();
        Ok(())
    }
}


pub struct WsIpcClient {
    url: url::Url,
    next_id: i64,
}

impl WsIpcClient {
    pub fn new(server_id: ::IpcServerId) -> Result<Self> {
        let url = url::Url::parse(&server_id).chain_err(|| "Unable to parse server_id as url")?;
        Ok(WsIpcClient { url, next_id: 1 })
    }

    pub fn call<T, O>(&mut self, method: &str, params: &T) -> Result<O>
        where T: serde::Serialize,
              O: for<'de> serde::Deserialize<'de>
    {
        let (result_tx, result_rx) = mpsc::channel();
        let factory = Factory {
            request: self.get_json(method, params),
            result_tx: result_tx,
        };
        let mut ws = ws::WebSocket::new(factory).chain_err(|| "Unable to create WebSocket")?;
        ws.connect(self.url.clone()).chain_err(|| "Unable to connect WebSocket to url")?;
        ws.run().chain_err(|| "Error while running WebSocket event loop")?;

        match result_rx.try_recv() {
            Ok(result) => result,
            Err(_) => bail!("Internal error, no WebSocket status"),
        }
    }

    fn get_json<T>(&mut self, method: &str, params: &T) -> String
        where T: serde::Serialize
    {
        let request_json = json!({
            "jsonrpc": "2.0",
            "id": self.get_id(),
            "method": method,
            "params": params,
        });
        format!("{}", request_json)
    }

    fn get_id(&mut self) -> i64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
}
