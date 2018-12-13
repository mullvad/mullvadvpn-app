use std::{marker::PhantomData, sync::mpsc};

/// Abstraction over an `mpsc::Sender` that first converts the value to another type before sending.
#[derive(Debug, Clone)]
pub struct IntoSender<T, U> {
    sender: mpsc::Sender<U>,
    _marker: PhantomData<T>,
}

impl<T, U> IntoSender<T, U>
where
    T: Into<U>,
{
    /// Converts the `T` into a `U` and sends it on the channel.
    pub fn send(&self, t: T) -> Result<(), mpsc::SendError<U>> {
        self.sender.send(t.into())
    }
}

impl<T, U> From<mpsc::Sender<U>> for IntoSender<T, U>
where
    T: Into<U>,
{
    fn from(sender: mpsc::Sender<U>) -> Self {
        IntoSender {
            sender,
            _marker: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{sync::mpsc, thread};

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
        let (tx, rx) = mpsc::channel::<Outer>();
        let inner_tx: IntoSender<Inner, Outer> = tx.clone().into();

        tx.send(Outer::Other).unwrap();
        inner_tx.send(Inner::Two).unwrap();

        assert_eq!(Outer::Other, rx.recv().unwrap());
        assert_eq!(Outer::Inner(Inner::Two), rx.recv().unwrap());
    }

    #[test]
    fn send_between_thread() {
        let (tx, rx) = mpsc::channel::<Outer>();
        let inner_tx: IntoSender<Inner, Outer> = tx.clone().into();

        thread::spawn(move || {
            inner_tx.send(Inner::One).unwrap();
        });

        assert_eq!(Outer::Inner(Inner::One), rx.recv().unwrap());
    }
}
