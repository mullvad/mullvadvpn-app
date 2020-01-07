use futures::sync::mpsc::{SendError, UnboundedSender};
use std::marker::PhantomData;

/// Abstraction over an `mpsc::Sender` that first converts the value to another type before sending.
#[derive(Debug, Clone)]
pub struct IntoSender<T, U> {
    sender: UnboundedSender<U>,
    _marker: PhantomData<T>,
}

impl<T, U> IntoSender<T, U>
where
    T: Into<U>,
{
    /// Converts the `T` into a `U` and sends it on the channel.
    pub fn send(&self, t: T) -> Result<(), SendError<U>> {
        self.sender.unbounded_send(t.into())
    }
}

impl<T, U> From<UnboundedSender<U>> for IntoSender<T, U>
where
    T: Into<U>,
{
    fn from(sender: UnboundedSender<U>) -> Self {
        IntoSender {
            sender,
            _marker: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::{sync::mpsc, Stream};
    use std::thread;

    #[derive(Debug, Eq, PartialEq)]
    enum Inner {
        One,
        Two,
    }

    #[derive(Debug, Eq, PartialEq)]
    enum Outer {
        Inner(Inner),
        Other,
    }

    impl From<Inner> for Outer {
        fn from(o: Inner) -> Self {
            Outer::Inner(o)
        }
    }

    #[test]
    fn sender() {
        let (tx, rx) = mpsc::unbounded();
        let inner_tx: IntoSender<Inner, Outer> = tx.clone().into();

        tx.unbounded_send(Outer::Other).unwrap();
        inner_tx.send(Inner::Two).unwrap();

        let mut sync_rx = rx.wait();

        assert_eq!(Outer::Other, sync_rx.next().unwrap().unwrap());
        assert_eq!(Outer::Inner(Inner::Two), sync_rx.next().unwrap().unwrap());
    }

    #[test]
    fn send_between_thread() {
        let (tx, rx) = mpsc::unbounded();
        let inner_tx: IntoSender<Inner, Outer> = tx.clone().into();

        thread::spawn(move || {
            inner_tx.send(Inner::One).unwrap();
        });

        let mut sync_rx = rx.wait();

        assert_eq!(Outer::Inner(Inner::One), sync_rx.next().unwrap().unwrap());
    }
}
