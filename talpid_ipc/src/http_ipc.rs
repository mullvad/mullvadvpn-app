extern crate tiny_http;
extern crate serde_json;

use super::{ErrorKind, Result, ResultExt, IpcServerId};
use serde;
use std::thread;

pub fn start_new_server<T, U, F>(on_message: F) -> Result<IpcServerId>
    where T: serde::Deserialize + 'static,
          U: serde::Serialize,
          F: FnMut(Result<T>) -> U + Send + 'static
{
    for port in 5000..5010 {
        let addr = format!("127.0.0.1:{}", port);

        if let Ok(server) = start_http_server(&addr) {
            let _ = start_receive_loop(server, on_message);
            debug!("Started a HTTP IPC server on {}", addr);
            return Ok(format!("http://{}", addr));
        }
    }

    bail!(ErrorKind::CouldNotStartServer)
}

fn start_http_server(addr: &str) -> Result<tiny_http::Server> {
    tiny_http::Server::http(addr).map_err(|e| ErrorKind::Msg(e.to_string()).into())
}

fn start_receive_loop<T, U, F>(server: tiny_http::Server, mut on_message: F)
    where T: serde::Deserialize + 'static,
          U: serde::Serialize,
          F: FnMut(Result<T>) -> U + Send + 'static
{
    thread::spawn(move || for mut request in server.incoming_requests() {
        let read_res = parse_request(&mut request);
        let response = on_message(read_res);
        let reply_res = send_response(&response, request);

        if let Err(e) = reply_res {
            error!("Failed sending reply to request, {}", e);
        }
    });
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