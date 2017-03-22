extern crate tiny_http;
extern crate serde_json;
extern crate hyper;

use super::{ErrorKind, Result, ResultExt, IpcServerId};
use serde;
use std::thread;

pub fn start_new_server<T, U, F>(on_message: F) -> Result<IpcServerId>
    where T: serde::Deserialize + 'static,
          U: serde::Serialize,
          F: FnMut(Result<T>) -> U + Send + 'static
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

fn start_receive_loop<T, U, F>(server: tiny_http::Server, mut on_message: F)
    where T: serde::Deserialize + 'static,
          U: serde::Serialize,
          F: FnMut(Result<T>) -> U + Send + 'static
{
    thread::spawn(move || for mut request in server.incoming_requests() {
        let read_res = parse_request(&mut request);
        let response = on_message(read_res);
        let reply_res = send_response(&response, request);

        if reply_res.is_err() {
            error!("Failed sending reply to request, {}",
                   reply_res.unwrap_err());
        }
    });
}

fn parse_request<T: serde::Deserialize>(request: &mut tiny_http::Request) -> Result<T> {
    serde_json::from_reader(request.as_reader()).chain_err(|| ErrorKind::ParseFailure)
}

fn send_response<U: serde::Serialize>(response: &U, request: tiny_http::Request) -> Result<()> {
    serde_json::to_string(response)
        .chain_err(|| ErrorKind::ReplyFailure)
        .and_then(|response_as_string| {

            debug!("HTTP IPC responding with {:?}", response_as_string);
            request.respond(tiny_http::Response::from_string(response_as_string))
                .chain_err(|| ErrorKind::ReplyFailure)
        })
}

pub struct IpcClient<T, U>
    where T: serde::Serialize,
          U: serde::Deserialize
{
    server_id: IpcServerId,
    client: hyper::Client,
    _phantom_t: ::std::marker::PhantomData<T>,
    _phantom_u: ::std::marker::PhantomData<U>,
}

impl<T, U> IpcClient<T, U>
    where T: serde::Serialize,
          U: serde::Deserialize
{
    pub fn new(server_id: IpcServerId) -> Self {
        IpcClient {
            server_id: server_id,
            client: hyper::Client::new(),
            _phantom_t: ::std::marker::PhantomData,
            _phantom_u: ::std::marker::PhantomData,
        }
    }

    pub fn send(&mut self, message: &T) -> Result<U> {
        let message_json = serde_json::to_string(message).chain_err(|| ErrorKind::ParseFailure)?;
        debug!("HTTP IPC sending {}", message_json);
        let mut reply = self.client
            .post(&self.server_id)
            .body(&message_json)
            .send()
            .chain_err(|| ErrorKind::SendError)?;


        Self::parse_reply(&mut reply)
    }

    fn parse_reply(reply: &mut hyper::client::response::Response) -> Result<U> {
        serde_json::from_reader(reply).chain_err(|| ErrorKind::ParseFailure)
    }
}
