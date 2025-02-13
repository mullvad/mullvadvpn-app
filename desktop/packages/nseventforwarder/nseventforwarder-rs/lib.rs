//! Forward [NSEvent]s from macOS to node.
#![cfg(target_os = "macos")]

use std::ptr::NonNull;
use std::sync::{mpsc, Arc, Mutex};
use std::thread::JoinHandle;

use block2::RcBlock;
use neon::prelude::{
    Context, FunctionContext, JsFunction, JsNull, JsResult, ModuleContext, NeonResult, Object, Root,
};
use objc2_app_kit::{NSEvent, NSEventMask};

#[neon::main]
fn main(mut cx: ModuleContext<'_>) -> NeonResult<()> {
    cx.export_function("start", start)?;
    Ok(())
}

/// Register a callback to fire every time a [NSEventMask::LeftMouseDown] or [NSEventMask::RightMouseDown] event occur.
///
/// Returns a stop function to call when the original callback shouldn't be called anymore. This
/// stop function returns a `true` value when called the first time and the callback is
/// deregistered. If it were to be called yet again, it will keep returning `false`.
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

    // NSEventForwarder instance. It must be cleaned up by the callback
    // function returned from `start` (aka `stop`). We use an Option here
    // because we can not enforce the Nodejs caller to only call `stop` once.
    let nseventforwarder = Mutex::new(Some(NSEventForwarder {
        thread: join_handle,
        stop: stop_tx,
    }));

    // Return a stop function to be invoked from the node runtime to deregister the NSEvent
    // callback.
    JsFunction::new(&mut cx, move |mut cx: FunctionContext<'_>| {
        // Stop this NSEventForwarder
        // Returns whether NSEventForwarder was stopped on this invocation of the stop function
        let mut stopped = false;
        if let Some(context) = nseventforwarder.lock().unwrap().take() {
            context.stop();
            stopped = true;
        }
        Ok(cx.boolean(stopped))
    })
}

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
