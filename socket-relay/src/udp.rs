use futures;
use futures::future::{self, Future};
use futures::sink::Sink;
use futures::stream::Stream;
use futures::sync::mpsc::{channel as sync_channel, Sender as SyncSender};
use futures::unsync::mpsc::{channel as unsync_channel, Sender};

use std::io;
use std::net::{IpAddr, SocketAddr};
use std::result::Result as StdResult;
use std::time::Duration;

use tokio_core::net::{UdpCodec, UdpSocket};
use tokio_core::reactor::Handle;
use tokio_timer::Timer;

/// The amount of idle (no replies) time needed for the forwarding socket to close.
pub static FORWARD_TIMEOUT_MS: u64 = 8000;

/// Number of slots in internal channel transfering responses back to clients.
pub static CLIENT_SINK_CHANNEL_SIZE: usize = 10;


pub struct Relay {
    listen_addr: SocketAddr,
    forwarding_future: Box<Future<Item = (), Error = io::Error>>,
    close_handle: SyncSender<()>,
}

impl Relay {
    /// Sets up relaying from `listen_addr` to `destination_addr`.
    ///
    /// `forward_bind_ip` is the local IP the socket that sends to the destination binds to.
    pub fn new(
        listen_addr: SocketAddr,
        forward_bind_ip: IpAddr,
        destination_addr: SocketAddr,
        handle: Handle,
    ) -> Result<Relay, io::Error> {
        let listen_socket = UdpSocket::bind(&listen_addr, &handle)?;
        let listen_addr = listen_socket.local_addr()?;
        debug!("Bound relay listening socket to {}", listen_addr);

        // Split the listening socket into a stream of incoming and a sink for outgoing datagrams.
        let (client_sink, client_stream) = listen_socket.framed(ServerCodec).split();
        let (closable_client_stream, close_handle) = closable_stream(client_stream);
        let client_sink_channel = create_client_sink_channel(client_sink, &handle);

        let forwarding_future = closable_client_stream.for_each(move |(client_addr, data)| {
            let response_sink = client_sink_channel.clone().sink_map_err(|_| ());

            if let Err(e) = Self::forward(
                client_addr,
                forward_bind_ip,
                destination_addr,
                data,
                response_sink,
                &handle,
            ) {
                error!("Unable to perform forwarding for {}: {}", client_addr, e);
            };
            future::ok(())
        });

        Ok(Relay {
            listen_addr,
            forwarding_future: Box::new(forwarding_future),
            close_handle,
        })
    }

    /// Forwards `data` to `destination` and streams all replies into `response_sink`.
    fn forward<S>(
        client_addr: SocketAddr,
        bind_ip: IpAddr,
        destination: SocketAddr,
        data: Vec<u8>,
        response_sink: S,
        handle: &Handle,
    ) -> io::Result<()>
    where
        S: Sink<SinkItem = (SocketAddr, Vec<u8>), SinkError = ()> + 'static,
    {
        let bind_addr = SocketAddr::new(bind_ip, 0);
        let socket = UdpSocket::bind(&bind_addr, &handle)?;
        trace!(
            "Relaying {} byte datagram from {} to {}",
            data.len(),
            client_addr,
            destination
        );

        let (forward_sink, forward_stream) = socket.framed(ServerCodec).split();

        let send_future = forward_sink.send((destination, data)).map_err(|e| {
            error!("Error while forwarding to destination addr: {}", e);
        });

        let recv_stream = forward_stream
            .filter_map(move |(addr, data)| {
                if addr == destination {
                    trace!(
                        "Returning {} byte response from {} to {}",
                        data.len(),
                        addr,
                        client_addr
                    );
                    Some((client_addr, data))
                } else {
                    trace!(
                        "Discarding data from {}, expecting data from {}",
                        addr,
                        destination
                    );
                    None
                }
            })
            .map_err(|e| {
                error!("Error reading datagrams from forward socket: {}", e)
            });

        let timeout_recv_future = Timer::default()
            .timeout_stream(recv_stream, Duration::from_millis(FORWARD_TIMEOUT_MS))
            .forward(response_sink)
            .map(|_| ());

        handle.spawn(send_future.and_then(|_| timeout_recv_future));
        Ok(())
    }

    pub fn listen_addr(&self) -> SocketAddr {
        self.listen_addr
    }

    pub fn close_handle(&self) -> RelayCloseHandle {
        RelayCloseHandle(self.close_handle.clone())
    }
}

impl Future for Relay {
    type Item = ();
    type Error = io::Error;

    fn poll(&mut self) -> futures::Poll<Self::Item, Self::Error> {
        self.forwarding_future.poll()
    }
}

impl Drop for Relay {
    fn drop(&mut self) {
        self.close_handle().close();
    }
}

pub struct RelayCloseHandle(SyncSender<()>);

impl RelayCloseHandle {
    pub fn close(self) {
        if self.0.send(()).wait().is_err() {
            warn!("Relay already closed");
        }
    }
}


fn closable_stream<S: Stream + 'static>(
    stream: S,
) -> (
    Box<Stream<Item = S::Item, Error = S::Error>>,
    SyncSender<()>,
) {
    let (close_handle_sink, close_signal_stream) = sync_channel(0);
    let close_signal_stream = close_signal_stream.map(|_| None).map_err(|_| None);
    let mapped_stream = stream.map(|t| Some(t)).map_err(|e| Some(e));

    let output_stream = mapped_stream
        .select(close_signal_stream)
        // Map close_signal_stream error as Ok(None) and stream error back to S::Error
        .then(|element| match element {
            Err(None) => Ok(None),
            Err(Some(e)) => Err(e),
            Ok(item) => Ok(item),
        })
        // Make the stream end when signaled by close_signal_stream.
        .take_while(|item| Ok(item.is_some()))
        // Map Option<S::Item> to S::Item, we know it is a Some from the take_while above.
        .map(|item| item.unwrap());
    (Box::new(output_stream), close_handle_sink)
}


/// Create a channel accepting tuples of `SocketAddr` and binary data and forward anything coming
/// on this channel to the `client_sink`. Returns the sender half of the channel.
fn create_client_sink_channel<S>(client_sink: S, handle: &Handle) -> Sender<(SocketAddr, Vec<u8>)>
where
    S: Sink<SinkItem = (SocketAddr, Vec<u8>), SinkError = io::Error> + 'static,
{
    let (channel_sink, channel_stream) = unsync_channel(CLIENT_SINK_CHANNEL_SIZE);

    let forward_future = channel_stream
        .map_err(|_| None)
        .forward(client_sink.sink_map_err(|e| Some(e)))
        .and_then(|_| Ok(()))
        .map_err(|error: Option<io::Error>| match error {
            Some(sink_error) => {
                error!("Error sending response back to client: {}", sink_error);
            }
            None => debug!("Closing relay socket sink"),
        });
    handle.spawn(forward_future);

    channel_sink
}


/// Internal struct implementing `Codec`. Just so it becomes possible to split a `UdpSocket` into
/// a sink and a stream.
struct ServerCodec;

impl UdpCodec for ServerCodec {
    type In = (SocketAddr, Vec<u8>);
    type Out = (SocketAddr, Vec<u8>);

    fn decode(&mut self, addr: &SocketAddr, buf: &[u8]) -> StdResult<Self::In, io::Error> {
        Ok((*addr, buf.to_vec()))
    }

    fn encode(&mut self, (addr, buf): Self::Out, into: &mut Vec<u8>) -> SocketAddr {
        into.extend(buf);
        addr
    }
}
