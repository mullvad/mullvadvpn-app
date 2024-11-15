//! Forward [NSEvent]s from macOS to node.
#![cfg(target_os = "macos")]
//#![warn(clippy::undocumented_unsafe_blocks)]

use neon::{prelude::{Context, FunctionContext, ModuleContext, NeonResult}, result::JsResult, types::JsPromise};

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("hasFullDiskAccess", has_full_disk_access)?;
    Ok(())
}

fn has_full_disk_access(mut cx: FunctionContext) -> JsResult<JsPromise> {
    let (deferred, promise) = cx.promise();
    let channel = cx.channel();

    std::thread::spawn(move || {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        let has_fda = runtime.block_on(talpid_macos::has_full_disk_access());

        //let fda = deferred.settle_with(channel, complete)
        deferred.settle_with(&channel, move |mut cx| {
            Ok(cx.boolean(has_fda))
        });
    });

    Ok(promise)
}
