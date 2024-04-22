use core::{hash::BuildHasher, mem::MaybeUninit, str::FromStr, time::Duration};
use std::{collections::HashMap, time::Instant};

use anyhow::{anyhow, bail};
use maybenot::{
    framework::{Framework, MachineId, TriggerEvent},
    machine::Machine,
};

/// A running Maybenot instance.
///
/// - Create it [ffi::maybenot_start].
/// - Feed it actions using [ffi::maybenot_on_event].
/// - Stop it using [ffi::maybenot_stop].
pub struct Maybenot {
    framework: Framework<Vec<Machine>>,

    // we aren't allowed to look into MachineId, so we need our own id type to use with FFI
    /// A map from `hash(MachineId)` to `MachineId`
    machine_id_hashes: HashMap<u64, MachineId>,
}

#[repr(C)]
#[derive(Debug)]
pub struct MaybenotEvent {
    pub event_type: MaybenotEventType,

    /// The number of bytes that was sent or received.
    pub xmit_bytes: u16,

    /// The ID of the machine that triggered the event, if any.
    pub machine: u64,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct MaybenotDuration {
    /// Number of whole seconds
    secs: u64,

    /// A nanosecond fraction of a second.
    nanos: u32,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum MaybenotEventType {
    /// We sent a normal packet.
    NonpaddingSent = 0,

    /// We received a normal packet.
    NonpaddingReceived = 1,

    /// We send a padding packet.
    PaddingSent = 2,

    /// We received a padding packet.
    PaddingReceived = 3,
}

#[repr(C, u32)]
#[derive(Debug, Clone, Copy)]
pub enum MaybenotAction {
    Cancel {
        /// The machine that generated the action.
        machine: u64,
    } = 0,

    /// Send a padding packet.
    InjectPadding {
        /// The machine that generated the action.
        machine: u64,

        /// The time to wait before injecting a padding packet.
        timeout: MaybenotDuration,

        replace: bool,
        bypass: bool,

        /// The size of the padding packet.
        size: u16,
    } = 1,

    BlockOutgoing {
        /// The machine that generated the action.
        machine: u64,

        /// The time to wait before blocking.
        timeout: MaybenotDuration,

        replace: bool,
        bypass: bool,

        /// How long to block.
        duration: MaybenotDuration,
    } = 2,
}

impl Maybenot {
    pub fn start(
        machines_str: &str,
        max_padding_bytes: f64,
        max_blocking_bytes: f64,
        mtu: u16,
    ) -> anyhow::Result<Self> {
        let machines: Vec<_> = machines_str
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .filter(|line| !line.starts_with('#'))
            .map(Machine::from_str)
            .collect::<Result<_, _>>()
            .map_err(|e| anyhow!("Failed to parse maybenot machine: {e}"))?;

        let framework = Framework::new(
            machines.clone(),
            max_padding_bytes,
            max_blocking_bytes,
            mtu,
            Instant::now(),
        )
        .map_err(|e| anyhow!("Failed to initialize framework: {e}"))?;

        Ok(Maybenot {
            framework,

            // TODO: consider using a faster hasher
            machine_id_hashes: Default::default(),
        })
    }

    pub fn on_event(
        &mut self,
        actions: &mut [MaybeUninit<MaybenotAction>],
        event: MaybenotEvent,
    ) -> anyhow::Result<u64> {
        let Some(event) = convert_event(event, &self.machine_id_hashes) else {
            bail!("Unknown machine");
        };

        let num_actions = self
            .framework
            .trigger_events(&[event], Instant::now())
            // convert maybenot actions to repr(C) equivalents
            .map(|action| convert_action(action, &mut self.machine_id_hashes))
            // write the actions to the out buffer
            .zip(actions.iter_mut())
            .map(|(action, out)| out.write(action))
            .count();

        Ok(num_actions as u64)
    }
}

fn hash_machine(machine_id: MachineId, machine_id_hashes: &mut HashMap<u64, MachineId>) -> u64 {
    let hash = machine_id_hashes.hasher().hash_one(machine_id);
    machine_id_hashes.insert(hash, machine_id);
    hash
}

fn machine_from_hash(hash: u64, machine_id_hashes: &HashMap<u64, MachineId>) -> Option<MachineId> {
    machine_id_hashes.get(&hash).copied()
}

/// Convert an action from [maybenot] to our own `repr(C)` action type.
fn convert_action(
    action: &maybenot::framework::Action,
    machine_id_hashes: &mut HashMap<u64, MachineId>,
) -> MaybenotAction {
    match *action {
        maybenot::framework::Action::Cancel { machine } => MaybenotAction::Cancel {
            machine: hash_machine(machine, machine_id_hashes),
        },
        maybenot::framework::Action::InjectPadding {
            timeout,
            size,
            bypass,
            replace,
            machine,
        } => MaybenotAction::InjectPadding {
            timeout: timeout.into(),
            size,
            replace,
            bypass,
            machine: hash_machine(machine, machine_id_hashes),
        },
        maybenot::framework::Action::BlockOutgoing {
            timeout,
            duration,
            bypass,
            replace,
            machine,
        } => MaybenotAction::BlockOutgoing {
            timeout: timeout.into(),
            duration: duration.into(),
            replace,
            bypass,
            machine: hash_machine(machine, machine_id_hashes),
        },
    }
}

fn convert_event(
    event: MaybenotEvent,
    machine_id_hashes: &HashMap<u64, MachineId>,
) -> Option<TriggerEvent> {
    Some(match event.event_type {
        MaybenotEventType::NonpaddingSent => TriggerEvent::NonPaddingSent {
            bytes_sent: event.xmit_bytes,
        },
        MaybenotEventType::NonpaddingReceived => TriggerEvent::NonPaddingRecv {
            bytes_recv: event.xmit_bytes,
        },
        MaybenotEventType::PaddingSent => TriggerEvent::PaddingSent {
            bytes_sent: event.xmit_bytes,
            machine: machine_from_hash(event.machine, machine_id_hashes)?,
        },
        MaybenotEventType::PaddingReceived => TriggerEvent::PaddingRecv {
            bytes_recv: event.xmit_bytes,
        },
    })
}

impl From<Duration> for MaybenotDuration {
    #[inline]
    fn from(duration: Duration) -> Self {
        MaybenotDuration {
            secs: duration.as_secs(),
            nanos: duration.subsec_nanos(),
        }
    }
}

pub mod ffi {
    use crate::{Maybenot, MaybenotAction, MaybenotEvent};
    use core::{ffi::CStr, mem::MaybeUninit, slice::from_raw_parts_mut};

    #[repr(u32)]
    pub enum MaybenotError {
        Ok = 0,
        MachineStringNotUtf8 = 1,
        InvalidMachineString = 2,
        StartFramework = 3,
        UnknownMachine = 4,
    }

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
    ) -> MaybenotError {
        // TODO: s/unwrap/error/
        // SAFETY: see function docs
        let out = unsafe { out.as_mut() }.unwrap();
        let machines_str = unsafe { CStr::from_ptr(machines_str) };
        let machines_str = machines_str.to_str().unwrap();

        let result = Maybenot::start(machines_str, max_padding_bytes, max_blocking_bytes, mtu);

        match result {
            Ok(maybenot) => {
                let box_pointer = Box::into_raw(Box::new(maybenot));
                out.write(box_pointer);

                MaybenotError::Ok
            }
            Err(_) => todo!("return maybenot_start error"),
        }
    }

    /// Get the number of machines running in the [Maybenot] instance.
    #[no_mangle]
    pub unsafe extern "C" fn maybenot_num_machines(this: *mut Maybenot) -> u64 {
        let this = unsafe { this.as_mut() }
            .expect("maybenot_num_machines expects a valid maybenot pointer");

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
    ) -> MaybenotError {
        let this =
            unsafe { this.as_mut() }.expect("maybenot_on_event expects a valid maybenot pointer");

        // TODO: Add safety reasoning
        let actions: &mut [MaybeUninit<MaybenotAction>] =
            unsafe { from_raw_parts_mut(actions_out, this.framework.num_machines()) };

        match this.on_event(actions, event) {
            Ok(num_actions) => {
                // TODO: Add safety reasoning
                unsafe { num_actions_out.write(num_actions) };
                MaybenotError::Ok
            }
            Err(_) => MaybenotError::UnknownMachine,
        }
    }
}
