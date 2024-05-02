use core::{ffi::c_void, hash::BuildHasher, str::FromStr, time::Duration};
use std::collections::HashMap;
use std::time::Instant;

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
    on_action: MaybenotActionCallback,
    framework: Framework<Vec<Machine>>,

    // we aren't allowed to look into MachineId, so we need our own id type to use with FFI
    /// A map from `hash(MachineId)` to `MachineId`
    machine_id_hashes: HashMap<u64, MachineId>,
}

/// A function that is called by [ffi::maybenot_on_event] once for every generated
/// [MaybenotAction].
// TODO: Consider passing an action buffer to `maybenot_on_event` instead of using a callback.
pub type MaybenotActionCallback = extern "C" fn(*mut c_void, MaybenotAction);

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
        on_action: MaybenotActionCallback,
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
            on_action,
            framework,

            // TODO: consider using a faster hasher
            machine_id_hashes: Default::default(),
        })
    }

    pub fn on_event(&mut self, user_data: *mut c_void, event: MaybenotEvent) -> anyhow::Result<()> {
        let Some(event) = convert_event(event, &self.machine_id_hashes) else {
            bail!("Unknown machine");
        };

        for action in self
            .framework
            .trigger_events(&[event.into()], Instant::now())
        {
            let action = convert_action(action, &mut self.machine_id_hashes);
            (self.on_action)(user_data, action)
        }

        Ok(())
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

fn convert_action(
    action: &maybenot::framework::Action,
    machine_id_hashes: &mut HashMap<u64, MachineId>,
) -> MaybenotAction {
    match action {
        &maybenot::framework::Action::Cancel { machine } => MaybenotAction::Cancel {
            machine: hash_machine(machine, machine_id_hashes),
        },
        &maybenot::framework::Action::InjectPadding {
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
        &maybenot::framework::Action::BlockOutgoing {
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
    use crate::{Maybenot, MaybenotActionCallback, MaybenotEvent};
    use std::ffi::{c_void, CStr};

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
    /// `machines_str` must be a null-terminated UTF-8 string, containing LF-separated machines.
    #[no_mangle]
    pub extern "C" fn maybenot_start(
        machines_str: *const i8,
        max_padding_bytes: f64,
        max_blocking_bytes: f64,
        mtu: u16,
        on_action: MaybenotActionCallback,
        out: *mut *mut Maybenot,
    ) -> MaybenotError {
        let machines_str = unsafe { CStr::from_ptr(machines_str) };
        let machines_str = machines_str.to_str().unwrap();

        let result = Maybenot::start(
            machines_str,
            max_padding_bytes,
            max_blocking_bytes,
            mtu,
            on_action,
        );

        match result {
            Ok(maybenot) => {
                let box_pointer = Box::into_raw(Box::new(maybenot));

                // SAFETY: caller pinky promises that `out` is a valid pointer.
                unsafe { out.write(box_pointer) };

                MaybenotError::Ok
            }
            Err(_) => todo!(),
        }
    }

    /// Stop a running [Maybenot] instance.
    ///
    /// This will free the maybenot pointer.
    #[no_mangle]
    pub extern "C" fn maybenot_stop(this: *mut Maybenot) {
        // Reconstruct the Box<Maybenot> and drop it.
        // SAFETY: caller pinky promises that this pointer was created by `maybenot_start`
        let _this = unsafe { Box::from_raw(this) };
    }

    /// Feed an event to the [Maybenot] instance.
    ///
    /// This may generate [super::MaybenotAction]s that will be sent to the callback provided to
    /// [maybenot_start]. `user_data` will be passed to the callback as-is, it will not be read or
    /// modified.
    #[no_mangle]
    pub extern "C" fn maybenot_on_event(
        this: *mut Maybenot,
        user_data: *mut c_void,
        event: MaybenotEvent,
    ) -> MaybenotError {
        let this =
            unsafe { this.as_mut() }.expect("maybenot_on_event expects a valid maybenot pointer");

        match this.on_event(user_data, event) {
            Ok(_) => MaybenotError::Ok,
            Err(_) => MaybenotError::UnknownMachine,
        }
    }
}
