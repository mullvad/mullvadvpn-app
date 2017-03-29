extern crate tiny_http;
extern crate serde_json;

use super::{ErrorKind, Result, ResultExt, IpcServerId};
use serde;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

pub struct HttpServerHandle {
    pub address: IpcServerId,
    stop_tx: mpsc::SyncSender<u8>,
}
impl HttpServerHandle {
    pub fn stop(&self) {
        let _ = self.stop_tx.send(0);
    }
}
impl Drop for HttpServerHandle {
    fn drop(&mut self) {
        self.stop();
    }
}

pub fn start_server<T, U, F>(on_message: F) -> Result<HttpServerHandle>
    where T: serde::Deserialize + 'static,
          U: serde::Serialize,
          F: FnMut(Result<T>) -> U + Send + 'static
{
    for port in 5000..5010 {
        let addr = format!("127.0.0.1:{}", port);

        if let Ok(server) = start_http_server(&addr) {
            let (stop_tx, stop_rx) = mpsc::sync_channel(0);
            let handle = HttpServerHandle {
                stop_tx: stop_tx,
                address: format!("http://{}", addr),
            };

            start_receive_loop(on_message, server, stop_rx);
            debug!("Started a HTTP IPC server on {}", addr);
            return Ok(handle);
        }
    }
    bail!(ErrorKind::CouldNotStartServer)
}

fn start_http_server(addr: &str) -> Result<tiny_http::Server> {
    tiny_http::Server::http(addr).map_err(|e| ErrorKind::Msg(e.to_string()).into())
}

fn start_receive_loop<T, U, F>(mut on_message: F,
                               http_server: tiny_http::Server,
                               stop_rx: mpsc::Receiver<u8>)
    where T: serde::Deserialize + 'static,
          U: serde::Serialize,
          F: FnMut(Result<T>) -> U + Send + 'static
{
    thread::spawn(move || {
        while !should_stop(&stop_rx) {
            receive(&mut on_message, &http_server);
        }
        debug!("Stopping the HTTP IPC server");
    });
}

fn should_stop(stop_rx: &mpsc::Receiver<u8>) -> bool {
    stop_rx.try_recv() != Err(mpsc::TryRecvError::Empty)
}

fn receive<T, U, F>(on_message: &mut F, http_server: &tiny_http::Server)
    where T: serde::Deserialize + 'static,
          U: serde::Serialize,
          F: FnMut(Result<T>) -> U + Send + 'static
{
    let req_res = http_server.recv_timeout(Duration::from_millis(1000));
    match req_res {
        Ok(Some(mut request)) => {
            let read_res = parse_request(&mut request);
            let response = on_message(read_res);
            let reply_res = send_response(&response, request);

            if let Err(e) = reply_res {
                error!("Failed sending reply to request, {}", e);
            }
        }
        Ok(None) => (),
        Err(e) => error!("Failed receiving request: {}", e),
    }
}

fn parse_request<T: serde::Deserialize>(request: &mut tiny_http::Request) -> Result<T> {
    let reader = request.as_reader();
    let mut buffer = String::new();
    reader.read_to_string(&mut buffer).chain_err(|| ErrorKind::ParseFailure)?;

    debug!("Got IPC request: {}", buffer);

    serde_json::from_str(&buffer).chain_err(|| ErrorKind::ParseFailure)
}

fn send_response<U: serde::Serialize>(response: &U, request: tiny_http::Request) -> Result<()> {
    serde_json::to_string(response)
        .chain_err(|| ErrorKind::ParseFailure)
        .and_then(|response_as_string| {

            debug!("HTTP IPC responding with {:?}", response_as_string);
            request.respond(tiny_http::Response::from_string(response_as_string))
                .chain_err(|| "Failed responding to HTTP request")
        })
}
