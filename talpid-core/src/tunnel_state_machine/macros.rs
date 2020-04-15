/// Try to receive an event from a `Stream`'s asynchronous poll expression.
///
/// This macro is similar to the `try_ready!` macro provided in `futures`. If there is an event
/// ready, it will be returned wrapped in a `Result`. If there are no events ready to be received,
/// the outer function will return with a transition that indicates that no events were received,
/// which is analogous to `Async::NotReady`.
///
/// When the asynchronous event indicates that the stream has finished or that it has failed, an
/// error type is returned so that either close scenario can be handled in a similar way.
macro_rules! try_handle_event {
    ($same_state:expr, $event:expr) => {
        match $event {
            Ok(futures01::Async::Ready(Some(event))) => Ok(event),
            Ok(futures01::Async::Ready(None)) => Err(None),
            Ok(futures01::Async::NotReady) => {
                return crate::tunnel_state_machine::EventConsequence::NoEvents($same_state);
            }
            Err(error) => Err(Some(error)),
        }
    };
}
