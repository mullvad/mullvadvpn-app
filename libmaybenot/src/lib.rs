use core::{ffi::c_void, hash::BuildHasher, str::FromStr, time::Duration};
use std::collections::HashMap;
use std::time::Instant;

use anyhow::{anyhow, bail};
use maybenot::{
    framework::{Framework, MachineId, TriggerEvent},
    machine::Machine,
};

pub struct Maybenot {
    on_action: ActionCallback,
    framework: Framework<Vec<Machine>>,

    // we aren't allowed to look into MachineId, so we need our own id type to use with FFI
    /// A map from `hash(MachineId)` to `MachineId`
    machine_id_hashes: HashMap<u64, MachineId>,
}

type ActionCallback = extern "C" fn(*mut c_void, Action);

pub const WIREGUARD_KEY_LENGTH: usize = 32;

#[repr(C)]
#[derive(Debug, Default)]
pub struct Event {
    pub event_type: EventType,
    pub xmit_bytes: u16,
    pub machine: u64,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CDuration {
    secs: u64,
    nanos: u32,
}
#[repr(C)]
#[derive(Debug, Default)]
#[allow(dead_code)]
pub enum EventType {
    #[default]
    NonpaddingSent,
    NonpaddingReceived,
    PaddingSent,
    PaddingReceived,
}

#[repr(C, u32)]
#[derive(Debug, Clone, Copy)]
pub enum Action {
    Cancel {
        machine: u64,
    } = 0,
    InjectPadding {
        machine: u64,
        timeout: CDuration,
        replace: bool,
        bypass: bool,
        size: u16,
    } = 1,
    BlockOutgoing {
        machine: u64,
        timeout: CDuration,
        replace: bool,
        bypass: bool,
        duration: CDuration,
    } = 2,
}

impl Maybenot {
    pub fn start(
        machines_str: &str,
        max_padding_bytes: f64,
        max_blocking_bytes: f64,
        mtu: u16,
        on_action: ActionCallback,
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

    pub fn on_event(&mut self, user_data: *mut c_void, event: Event) -> anyhow::Result<()> {
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
) -> Action {
    match action {
        &maybenot::framework::Action::Cancel { machine } => Action::Cancel {
            machine: hash_machine(machine, machine_id_hashes),
        },
        &maybenot::framework::Action::InjectPadding {
            timeout,
            size,
            bypass,
            replace,
            machine,
        } => Action::InjectPadding {
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
        } => Action::BlockOutgoing {
            timeout: timeout.into(),
            duration: duration.into(),
            replace,
            bypass,
            machine: hash_machine(machine, machine_id_hashes),
        },
    }
}

fn convert_event(
    event: Event,
    machine_id_hashes: &HashMap<u64, MachineId>,
) -> Option<TriggerEvent> {
    Some(match event.event_type {
        EventType::NonpaddingSent => TriggerEvent::NonPaddingSent {
            bytes_sent: event.xmit_bytes,
        },
        EventType::NonpaddingReceived => TriggerEvent::NonPaddingRecv {
            bytes_recv: event.xmit_bytes,
        },
        EventType::PaddingSent => TriggerEvent::PaddingSent {
            bytes_sent: event.xmit_bytes,
            machine: machine_from_hash(event.machine, machine_id_hashes)?,
        },
        EventType::PaddingReceived => TriggerEvent::PaddingRecv {
            bytes_recv: event.xmit_bytes,
        },
    })
}

impl From<Duration> for CDuration {
    #[inline]
    fn from(duration: Duration) -> Self {
        CDuration {
            secs: duration.as_secs(),
            nanos: duration.subsec_nanos(),
        }
    }
}

pub mod ffi {
    use crate::{ActionCallback, Event, Maybenot};
    use std::ffi::{c_void, CStr};

    #[repr(u32)]
    pub enum Error {
        Ok = 0,
        MachineStringNotUtf8 = 1,
        InvalidMachineString = 2,
        StartFramework = 3,
        UnknownMachine = 4,
    }

    #[no_mangle]
    pub extern "C" fn maybenot_start(
        machines_str: *const i8,
        max_padding_bytes: f64,
        max_blocking_bytes: f64,
        mtu: u16,
        on_action: ActionCallback,
        out: *mut *mut Maybenot,
    ) -> Error {
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

                Error::Ok
            }
            Err(_) => todo!(),
        }
    }

    #[no_mangle]
    pub extern "C" fn maybenot_stop(this: *mut Maybenot) {
        // Reconstruct the Box<Maybenot> and drop it.
        // SAFETY: caller pinky promises that this pointer was created by `maybenot_start`
        let _this = unsafe { Box::from_raw(this) };
    }

    #[no_mangle]
    pub extern "C" fn maybenot_on_event(
        this: *mut Maybenot,
        user_data: *mut c_void,
        event: Event,
    ) -> Error {
        let this =
            unsafe { this.as_mut() }.expect("maybenot_on_event expects a valid maybenot pointer");

        match this.on_event(user_data, event) {
            Ok(_) => Error::Ok,
            Err(_) => Error::UnknownMachine,
        }
    }
}
