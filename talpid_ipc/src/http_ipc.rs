extern crate tiny_http;
extern crate serde_json;
extern crate hyper;

use super::{ErrorKind, Result, ResultExt, IpcServerId};
use serde;
use std::thread;

pub fn start_new_server<T, F>(on_message: F) -> Result<IpcServerId>
    where T: serde::Deserialize + 'static,
          F: FnMut(Result<T>) + Send + 'static
{
    let addr = "127.0.0.1:5000";

    tiny_http::Server::http(addr)
        .map_err(|e| chain_boxed_err(e, ErrorKind::CouldNotStartServer))
        .and_then(|server| {
            start_receive_loop(server, on_message);
            debug!("Started a HTTP IPC server on {}", addr);
            Ok(format!("http://{}", addr))
        })
}

fn chain_boxed_err(boxed_cause: Box<::std::error::Error>, new_error: ErrorKind) -> super::Error {
    let cause = super::Error::from_kind(ErrorKind::Msg(boxed_cause.to_string()));
    super::Error::with_chain(cause, new_error)
}

fn start_receive_loop<T, F>(server: tiny_http::Server, mut on_message: F)
    where T: serde::Deserialize + 'static,
          F: FnMut(Result<T>) + Send + 'static
{
    thread::spawn(move || for mut request in server.incoming_requests() {
        let read_res = read_body_as_string(&mut request)
            .and_then(|s| serde_json::from_str(&s).chain_err(|| ErrorKind::ParseFailure));

        on_message(read_res);

        let _ = request.respond(tiny_http::Response::from_string("Ok"));
    });
}

fn read_body_as_string(request: &mut tiny_http::Request) -> Result<String> {
    let mut buffer = Vec::new();
    request.as_reader()
        .read_to_end(&mut buffer)
        .chain_err(|| ErrorKind::ReadFailure)?;

    let res = String::from_utf8(buffer).chain_err(|| ErrorKind::ReadFailure);
    debug!("HTTP IPC read body {:?}", res);
    res
}

pub struct IpcClient<T>
    where T: serde::Serialize
{
    server_id: IpcServerId,
    client: hyper::Client,
    _phantom: ::std::marker::PhantomData<T>,
}

impl<T> IpcClient<T>
    where T: serde::Serialize
{
    pub fn new(server_id: IpcServerId) -> Self {
        IpcClient {
            server_id: server_id,
            client: hyper::Client::new(),
            _phantom: ::std::marker::PhantomData,
        }
    }

    pub fn send(&mut self, message: &T) -> Result<()> {
        let message_json = serde_json::to_string(message).chain_err(|| ErrorKind::ParseFailure)?;
        debug!("HTTP IPC sending {}", message_json);
        self.client
            .post(&self.server_id)
            .body(&message_json)
            .send()
            .map(|_| ())
            .chain_err(|| ErrorKind::SendError)
    }
}
