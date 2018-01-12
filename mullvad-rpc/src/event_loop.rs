use std::thread;
use tokio_core::reactor::Core;

error_chain! {
    errors {
        CoreError { description("Error when creating event loop") }
    }
}

/// Creates a new tokio event loop on a new thread, runs the provided `init` closure on the thread
/// and sends back the result.
/// Used to spawn futures on the core in the separate thread and be able to return sendable handles.
pub fn create<F, T>(init: F) -> Result<T>
where
    F: FnOnce(&mut Core) -> T + Send + 'static,
    T: Send + 'static,
{
    let (tx, rx) = ::std::sync::mpsc::channel();
    thread::spawn(move || match create_core(init) {
        Err(e) => tx.send(Err(e)).unwrap(),
        Ok((mut core, out)) => {
            tx.send(Ok(out)).unwrap();
            loop {
                core.turn(None);
            }
        }
    });
    rx.recv().unwrap()
}

fn create_core<F, T>(init: F) -> Result<(Core, T)>
where
    F: FnOnce(&mut Core) -> T + Send + 'static,
{
    let mut core = Core::new().chain_err(|| ErrorKind::CoreError)?;
    let out = init(&mut core);
    Ok((core, out))
}
