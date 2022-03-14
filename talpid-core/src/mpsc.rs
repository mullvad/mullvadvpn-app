/// Abstraction over any type that can be used similarly to an `std::mpsc::Sender`.
pub trait Sender<T> {
    /// Sends an item over the underlying channel, failing only if the channel is closed.
    fn send(&self, item: T) -> Result<(), ()>;
}

impl<E> Sender<E> for futures::channel::mpsc::UnboundedSender<E> {
    fn send(&self, content: E) -> Result<(), ()> {
        self.unbounded_send(content).map_err(|_| ())
    }
}
