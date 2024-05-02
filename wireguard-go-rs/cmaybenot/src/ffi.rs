use crate::error::MaybenotResult;
use crate::{Maybenot, MaybenotAction, MaybenotEvent};
use std::{ffi::CStr, mem::MaybeUninit, slice::from_raw_parts_mut};

/// Start a new [Maybenot] instance.
///
/// # Safety
/// - `machines_str` must be a null-terminated UTF-8 string, containing LF-separated machines.
/// - `out` must be a valid pointer to some valid pointer-sized memory.
#[no_mangle]
pub unsafe extern "C" fn maybenot_start(
    machines_str: *const i8,
    max_padding_bytes: f64,
    max_blocking_bytes: f64,
    mtu: u16,
    out: *mut MaybeUninit<*mut Maybenot>,
) -> MaybenotResult {
    // SAFETY: see function docs
    let Some(out) = (unsafe { out.as_mut() }) else {
        return MaybenotResult::NullPointer;
    };

    // SAFETY: see function docs
    let machines_str = unsafe { CStr::from_ptr(machines_str) };
    let Ok(machines_str) = machines_str.to_str() else {
        return MaybenotResult::MachineStringNotUtf8;
    };

    Maybenot::start(machines_str, max_padding_bytes, max_blocking_bytes, mtu)
        .map(|maybenot| {
            let box_pointer = Box::into_raw(Box::new(maybenot));
            out.write(box_pointer);
        })
        .into()
}

/// Get the number of machines running in the [Maybenot] instance.
#[no_mangle]
pub unsafe extern "C" fn maybenot_num_machines(this: *mut Maybenot) -> u64 {
    let this =
        unsafe { this.as_mut() }.expect("maybenot_num_machines expects a valid maybenot pointer");

    this.framework.num_machines() as u64
}

/// Stop a running [Maybenot] instance. This will free the maybenot pointer.
///
/// # Safety
/// The pointer MUST have been created by [maybenot_start].
#[no_mangle]
pub unsafe extern "C" fn maybenot_stop(this: *mut Maybenot) {
    // Reconstruct the Box<Maybenot> and drop it.
    // SAFETY: caller pinky promises that this pointer was created by `maybenot_start`
    let _this = unsafe { Box::from_raw(this) };
}

/// Feed an event to the [Maybenot] instance.
///
/// This may generate [super::MaybenotAction]s that will be written to `actions_out`,
/// which must have a capacity at least equal to [maybenot_num_machines].
///
/// The number of actions will be written to `num_actions_out`.
#[no_mangle]
pub unsafe extern "C" fn maybenot_on_event(
    this: *mut Maybenot,
    event: MaybenotEvent,

    actions_out: *mut MaybeUninit<MaybenotAction>,
    num_actions_out: *mut u64,
) -> MaybenotResult {
    let Some(this) = (unsafe { this.as_mut() }) else {
        return MaybenotResult::NullPointer;
    };

    // SAFETY: called promises that `actions_out` points to valid memory with the capacity to
    // hold at least a `num_machines` amount of `MaybenotAction`. Rust arrays have the same
    // layout as C arrays. Since we use `MaybeUninit`, rust won't assume that the slice
    // elements have been initialized.
    let actions: &mut [MaybeUninit<MaybenotAction>] =
        unsafe { from_raw_parts_mut(actions_out, this.framework.num_machines()) };

    match this.on_event(actions, event) {
        Ok(num_actions) => {
            unsafe { num_actions_out.write(num_actions) };
            MaybenotResult::Ok
        }
        Err(_) => MaybenotResult::UnknownMachine,
    }
}
