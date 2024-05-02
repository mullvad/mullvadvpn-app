pub mod error;
pub mod ffi;

use core::{hash::BuildHasher, mem::MaybeUninit, str::FromStr, time::Duration};
use std::{collections::HashMap, time::Instant};

use maybenot::{
    framework::{Framework, MachineId, TriggerEvent},
    machine::Machine,
};

use crate::error::Error;

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
    ) -> Result<Self, Error> {
        let machines: Vec<_> = machines_str
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .filter(|line| !line.starts_with('#'))
            .map(Machine::from_str)
            .collect::<Result<_, _>>()
            .map_err(|_e| Error::InvalidMachineString)?;

        let framework = Framework::new(
            machines.clone(),
            max_padding_bytes,
            max_blocking_bytes,
            mtu,
            Instant::now(),
        )
        .map_err(|_e| Error::StartFramework)?;

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
    ) -> Result<u64, Error> {
        let event = convert_event(event, &self.machine_id_hashes)?;

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

fn machine_from_hash(
    hash: u64,
    machine_id_hashes: &HashMap<u64, MachineId>,
) -> Result<MachineId, Error> {
    machine_id_hashes
        .get(&hash)
        .copied()
        .ok_or(Error::UnknownMachine)
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
) -> Result<TriggerEvent, Error> {
    Ok(match event.event_type {
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
