/// Abstraction over any type that can be used similarly to an `std::mpsc::Sender`.
pub trait Sender<T> {
    /// Sends an item over the underlying channel, failing only if the channel is closed.
    fn send(&self, item: T) -> Result<(), ()>;
}
