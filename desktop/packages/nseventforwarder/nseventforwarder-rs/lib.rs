//! Forward [NSEvent]s from macOS to node.
#![cfg(target_os = "macos")]
#![warn(clippy::undocumented_unsafe_blocks)]

use std::ptr::NonNull;
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
fn main(mut cx: ModuleContext<'_>) -> NeonResult<()> {
    cx.export_function("start", start)?;
    Ok(())
}

/// NSEventForwarder instance. It must be initialized by `start` and cleaned up by the callback
/// function returned from `start`.
static NSEVENTFORWARDER: Mutex<Option<NSEventForwarder>> = Mutex::new(None);

struct NSEventForwarder {
    /// The thread listening for incoming [NSEvent]s.
    thread: JoinHandle<()>,
    /// Signal for the current execution context to stop.
    stop: mpsc::Sender<()>,
}

impl NSEventForwarder {
    fn stop(self) {
        // Tell the thread to stop running
        let _ = self.stop.send(());
        // Wait for the thread to shutdown
        self.thread
            .join()
            .expect("Couldn't join the NSEventForwarder thread");
    }
}

/// Register a callback to fire every time a [NSEventMask::LeftMouseDown] or [NSEventMask::RightMouseDown] event occur.
///
/// Returns a stop function to call when the original callback shouldn't be called anymore.
fn start(mut cx: FunctionContext<'_>) -> JsResult<'_, JsFunction> {
    // Set up neon stuff.
    // These will be moved into the spawned thread
    let nodejs_callback = cx.argument::<JsFunction>(0)?.root(&mut cx);
    let channel = cx.channel();
    // Start a long-running thread which handles incoming NS events
    // When a new event is received, call the callback passed to us from the JavaScript caller
    let (stop_tx, stop_rx) = mpsc::channel();
    let join_handle = std::thread::spawn(move || {
        // Each time the nodejs callback is triggered, we need to reference the nodejs
        // function reference. As such, we keep it from being garbage-collected with the
        // Root handle type and allow sharing it with the RCBlock via an Arc.
        let nodesjs_callback: Arc<Root<JsFunction>> = Arc::new(nodejs_callback);
        // Create a callback which will be called on the registered NSEvents.
        // When called schedules a closure to execute in nodejs thread that invoked start.
        let nsevent_callback = move |_nsevent: NonNull<NSEvent>| {
            let nodejs_callback = Arc::clone(&nodesjs_callback);
            channel.send(move |mut cx| {
                let this = JsNull::new(&mut cx);
                let _ = nodejs_callback.to_inner(&mut cx).call(&mut cx, this, []);
                Ok(())
            });
        };
        // Start monitoring incoming NS events
        // SAFETY: This function is trivially safe to call.
        // Note: Make sure to cancel this handler with [NSEvent::removeMonitor] to unregister the
        // listener.
        let mut handler = unsafe {
            NSEvent::addGlobalMonitorForEventsMatchingMask_handler(
                NSEventMask::LeftMouseDown | NSEventMask::RightMouseDown,
                &RcBlock::new(nsevent_callback),
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

    let new_context = NSEventForwarder {
        thread: join_handle,
        stop: stop_tx,
    };

    // Update the global NSEventForwarder state
    let mut nseventmonitor_context = NSEVENTFORWARDER.lock().unwrap();
    // Stop any old NSEventForwarder
    if let Some(context) = nseventmonitor_context.take() {
        context.stop();
    }
    let _ = nseventmonitor_context.insert(new_context);
    drop(nseventmonitor_context);

    // Return a stop function to be invoked from the node runtime to deregister the NSEvent
    // callback.
    JsFunction::new(&mut cx, stop)
}

fn stop(mut cx: FunctionContext<'_>) -> Result<Handle<'_, JsUndefined>, Throw> {
    if let Some(context) = NSEVENTFORWARDER.lock().unwrap().take() {
        context.stop();
    }

    Ok(JsUndefined::new(&mut cx))
}
