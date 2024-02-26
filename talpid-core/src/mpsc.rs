/// Error type for `Sender` trait.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// The underlying channel is closed.
    #[error("Channel is closed")]
    ChannelClosed,
}

/// Abstraction over any type that can be used similarly to an `std::mpsc::Sender`.
pub trait Sender<T> {
    /// Sends an item over the underlying channel, failing only if the channel is closed.
    fn send(&self, item: T) -> Result<(), Error>;
}

impl<E> Sender<E> for futures::channel::mpsc::UnboundedSender<E> {
    fn send(&self, content: E) -> Result<(), Error> {
        self.unbounded_send(content)
            .map_err(|_| Error::ChannelClosed)
    }
}
