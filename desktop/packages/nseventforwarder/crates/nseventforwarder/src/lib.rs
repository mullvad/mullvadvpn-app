//! Forward [NSEvent]s from macOS to node.
#![cfg(target_os = "macos")]
#![warn(clippy::undocumented_unsafe_blocks)]

use std::sync::{mpsc, Arc, Mutex};
use std::thread::JoinHandle;

use block2::RcBlock;
use neon::prelude::{
    Context, FunctionContext, Handle, JsFunction, JsNull, JsResult, JsUndefined, ModuleContext,
    NeonResult, Object, Root,
};
use neon::result::Throw;
use objc2_app_kit::{NSEvent, NSEventMask};

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("start", start)?;
    Ok(())
}

/// NSEventMonitor instance. It must be initialized by `start` and cleaned up by the callback
/// function returned from `start`.
static NSEVENTMONITOR: Mutex<Option<NSEventMonitor>> = Mutex::new(None);

struct NSEventMonitor {
    /// The thread listening for incoming [NSEvent]s.
    thread: JoinHandle<()>,
    /// Signal for the current execution context to stop.
    stop: mpsc::Sender<()>,
}

/// Register a callback to fire every time a [NSEventMask::LeftMouseDown] or [NSEventMask::RightMouseDown] event occur.
///
/// Returns a stop function to call when the original callback shouldn't be called anymore.
fn start(mut cx: FunctionContext) -> JsResult<JsFunction> {
    // Set up neon stuff
    let callback = cx.argument::<JsFunction>(0)?.root(&mut cx);
    let callback: Arc<Root<JsFunction>> = Arc::new(callback);
    let channel = cx.channel();

    // Start a long-running thread which handles incoming NS events
    // When a new event is received, call the callback passed to us from the JavaScript caller
    let (stop_tx, stop_rx) = mpsc::channel();
    let join_handle = std::thread::spawn(move || {
        let callback = Arc::<Root<JsFunction>>::clone(&callback);
        // Scaffolding for calling the JavaScript callback function
        let call_callback = move || {
            let cb = Arc::clone(&callback);
            channel.send(move |mut cx| {
                let this = JsNull::new(&mut cx);
                let _ = cb.to_inner(&mut cx).call(&mut cx, this, []);
                Ok(())
            })
        };
        // Start monitoring incoming NS events
        let block = RcBlock::new(move |_event| {
            call_callback();
        });
        // SAFETY: This function is trivially safe to call.
        // Note: Make sure to cancel this handler with [NSEvent::removeMonitor] to unregister the
        // listener.
        let mut handler = unsafe {
            NSEvent::addGlobalMonitorForEventsMatchingMask_handler(
                NSEventMask::LeftMouseDown | NSEventMask::RightMouseDown,
                &block,
            )
        };

        // Listen for stop signal
        let _ = stop_rx.recv();
        if let Some(handler) = handler.take() {
            // SAFETY: handler is removed only once.
            // See https://developer.apple.com/documentation/appkit/nsevent/1533709-removemonitor#discussion
            unsafe { NSEvent::removeMonitor(&handler) }
        }
        // The thread's execution will stop when this function returns
    });

    let new_context = NSEventMonitor {
        thread: join_handle,
        stop: stop_tx,
    };

    // Update the global NSEventMonitor state
    let mut nseventmonitor_context = NSEVENTMONITOR.lock().unwrap();
    // Stop any old NSEventMonitor
    if let Some(context) = nseventmonitor_context.take() {
        let _ = context.stop.send(());
        context
            .thread
            .join()
            .expect("Couldn't join the NSEventMonitor thread");
    }
    let _ = nseventmonitor_context.insert(new_context);
    drop(nseventmonitor_context);

    JsFunction::new(&mut cx, stop)
}

fn stop(mut cx: FunctionContext<'_>) -> Result<Handle<'_, JsUndefined>, Throw> {
    if let Some(context) = NSEVENTMONITOR.lock().unwrap().take() {
        // Tell the thread to stop running
        let _ = context.stop.send(());
        // Wait for the thread to shutdown
        context
            .thread
            .join()
            .expect("Couldn't join the NSEventMonitor thread");
    }

    Ok(JsUndefined::new(&mut cx))
}
