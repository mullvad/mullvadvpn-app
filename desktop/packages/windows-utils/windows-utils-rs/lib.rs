#![cfg(target_os = "windows")]

use neon::{prelude::ModuleContext, result::NeonResult};

mod shortcut;

#[neon::main]
fn main(mut cx: ModuleContext<'_>) -> NeonResult<()> {
    cx.export_function("readShortcut", shortcut::read_shortcut)?;

    Ok(())
}
