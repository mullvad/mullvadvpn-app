use futures::{Async, Future, Poll, Stream};
use std::time;
use tokio_timer::Timer;

/// A stream that only returns the last item that was received in a given time period.
pub struct GracefulStream<S>
where
    S: Stream,
{
    timer: Timer,
    timeout: time::Duration,
    stream: Option<S>,
    delayed_item: Option<DelayedValue<S::Item>>,
}

impl<S: Stream> GracefulStream<S> {
    pub fn new(stream: S, timeout: time::Duration) -> Self {
        let timer = Default::default();
        Self {
            timer,
            timeout,
            stream: Some(stream),
            delayed_item: None,
        }
    }

    fn poll_delay(&mut self) -> Async<Option<S::Item>> {
        match self.delayed_item.as_mut() {
            Some(value) => match value.poll() {
                Ok(Async::Ready(value)) => {
                    self.delayed_item = None;
                    Async::Ready(Some(value))
                }
                Ok(Async::NotReady) => Async::NotReady,
                Err(_) => Async::Ready(None),
            },
            None => Async::NotReady,
        }
    }
}

impl<S: Stream> Stream for GracefulStream<S> {
    type Item = S::Item;
    type Error = S::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        if let Some(mut stream) = self.stream.take() {
            let mut next_item = None;
            loop {
                match stream.poll()? {
                    Async::Ready(Some(item)) => {
                        next_item = Some(item);
                    }
                    Async::Ready(None) => {
                        return Ok(Async::Ready(
                            self.delayed_item.take().and_then(|value| value.consume()),
                        ));
                    }
                    Async::NotReady => break,
                }
            }
            self.stream = Some(stream);
            if let Some(next_item) = next_item {
                self.delayed_item =
                    Some(DelayedValue::new(next_item, self.timer.sleep(self.timeout)));
            }
        };
        Ok(self.poll_delay())
    }
}

struct DelayedValue<I> {
    delay: tokio_timer::Sleep,
    value: Option<I>,
}

impl<I> DelayedValue<I> {
    fn consume(mut self) -> Option<I> {
        self.value.take()
    }
}

impl<I> Future for DelayedValue<I> {
    type Item = I;
    type Error = ();

    fn poll(&mut self) -> Poll<I, ()> {
        match self
            .delay
            .poll()
            .map_err(|e| log::error!("Timer error: {}", e))?
        {
            Async::NotReady => Ok(Async::NotReady),
            Async::Ready(_) => Ok(Async::Ready(self.value.take().unwrap())),
        }
    }
}

impl<I> DelayedValue<I> {
    fn new(value: I, delay: tokio_timer::Sleep) -> Self {
        Self {
            value: Some(value),
            delay,
        }
    }
}
