use super::WIREGUARD_KEY_LENGTH;
use maybenot::{MachineId, Timer};
use once_cell::sync::OnceCell;
use rand::{
    rngs::{adapter::ReseedingRng, OsRng},
    SeedableRng,
};
use std::{
    collections::HashMap, fs, io, os::windows::prelude::RawHandle, path::Path, sync::Arc,
    time::Duration,
};
use talpid_types::net::wireguard::PublicKey;
use tokio::task::JoinHandle;
use windows_sys::Win32::{
    Foundation::{BOOLEAN, ERROR_NO_MORE_ITEMS},
    System::Threading::{WaitForMultipleObjects, WaitForSingleObject, INFINITE},
};

type Rng = ReseedingRng<rand_chacha::ChaCha12Core, OsRng>;
const RNG_RESEED_THRESHOLD: u64 = 1024 * 64; // 64 KiB

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Failed to find maybenot machines
    #[error("Failed to enumerate maybenot machines")]
    EnumerateMachines(#[source] io::Error),
    /// Failed to parse maybenot machine
    #[error("Failed to parse maybenot machine \"{0}\"")]
    InvalidMachine(String),
    /// Failed to initialize quit event
    #[error("Failed to initialize quit event")]
    InitializeQuitEvent(#[source] io::Error),
    /// Failed to initialize machinist handle
    #[error("Failed to initialize machinist handle")]
    InitializeHandle(#[source] io::Error),
    /// Failed to initialize maybenot framework
    #[error("Failed to initialize maybenot framework: {0}")]
    InitializeMaybenot(String),
}

// See DAITA_EVENT_TYPE:
// https://github.com/mullvad/wireguard-nt-priv/blob/mullvad-patches/driver/daita.h
#[repr(C)]
#[derive(Debug)]
#[allow(dead_code)]
pub enum EventType {
    NonpaddingSent,
    NonpaddingReceived,
    PaddingSent,
    PaddingReceived,
}

// See DAITA_EVENT:
// https://github.com/mullvad/wireguard-nt-priv/blob/mullvad-patches/driver/daita.h
#[repr(C)]
#[derive(Debug)]
pub struct Event {
    pub peer: [u8; WIREGUARD_KEY_LENGTH],
    pub event_type: EventType,
    pub xmit_bytes: u16,
    pub user_context: usize,
}

// See DAITA_ACTION_TYPE:
// https://github.com/mullvad/wireguard-nt-priv/blob/mullvad-patches/driver/daita.h
#[repr(C)]
pub enum ActionType {
    InjectPadding,
}

// See DAITA_PADDING_ACTION:
// https://github.com/mullvad/wireguard-nt-priv/blob/mullvad-patches/driver/daita.h
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct PaddingAction {
    pub byte_count: u16,
    pub replace: BOOLEAN,
}

// See DAITA_ACTION:
// https://github.com/mullvad/wireguard-nt-priv/blob/mullvad-patches/driver/daita.h
#[repr(C)]
pub struct Action {
    pub peer: [u8; WIREGUARD_KEY_LENGTH],
    pub action_type: ActionType,
    pub payload: ActionPayload,
    pub user_context: usize,
}

#[repr(C)]
pub union ActionPayload {
    pub padding: PaddingAction,
}

/// Maximum number of events that can be stored in the underlying buffer
const EVENTS_CAPACITY: usize = 1000;
/// Maximum number of actions that can be stored in the underlying buffer
const ACTIONS_CAPACITY: usize = 1000;

pub mod bindings {
    use super::*;
    use windows_sys::Win32::Foundation::BOOL;

    pub type WireGuardDaitaActivateFn = unsafe extern "stdcall" fn(
        adapter: RawHandle,
        events_capacity: usize,
        actions_capacity: usize,
    ) -> BOOL;
    pub type WireGuardDaitaEventDataAvailableEventFn =
        unsafe extern "stdcall" fn(adapter: RawHandle) -> RawHandle;
    pub type WireGuardDaitaReceiveEventsFn =
        unsafe extern "stdcall" fn(adapter: RawHandle, events: *mut Event) -> usize;
    pub type WireGuardDaitaSendActionFn =
        unsafe extern "stdcall" fn(adapter: RawHandle, action: *const Action) -> BOOL;
}

#[derive(Debug)]
pub struct Session {
    adapter: Arc<super::WgNtAdapter>,
}

impl Session {
    /// Call `WireGuardDaitaActivate` for an existing WireGuard interface
    pub(super) fn from_adapter(adapter: Arc<super::WgNtAdapter>) -> io::Result<Session> {
        // SAFETY: `WgNtAdapter` has a valid adapter handle
        unsafe {
            adapter
                .dll_handle
                .daita_activate(adapter.handle, EVENTS_CAPACITY, ACTIONS_CAPACITY)
        }?;
        Ok(Self { adapter })
    }

    pub fn receive_events<'a>(
        &self,
        buffer: &'a mut [Event; EVENTS_CAPACITY],
    ) -> io::Result<&'a [Event]> {
        let num_events = unsafe {
            // SAFETY: The adapter is valid, and the buffer is large enough to accommodate all
            // events.
            self.adapter
                .dll_handle
                .daita_receive_events(self.adapter.handle, buffer.as_mut_ptr())?
        };
        Ok(unsafe { std::slice::from_raw_parts(buffer.as_ptr(), num_events) })
    }

    pub fn send_action(&self, action: &Action) -> io::Result<()> {
        // SAFETY: The adapter is valid
        unsafe {
            self.adapter
                .dll_handle
                .daita_send_action(self.adapter.handle, action)
        }
    }

    pub fn event_data_available_event(&self) -> RawHandle {
        // SAFETY: The adapter is valid
        // This never fails when there's a DAITA session
        unsafe {
            self.adapter
                .dll_handle
                .daita_event_data_available_event(self.adapter.handle)
                .unwrap()
        }
    }
}

fn maybenot_event_from_event(
    event: &Event,
    machine_ids: &MachineMap,
) -> Option<maybenot::TriggerEvent> {
    match event.event_type {
        EventType::PaddingReceived => Some(maybenot::TriggerEvent::PaddingRecv),
        EventType::NonpaddingSent => Some(maybenot::TriggerEvent::NormalSent),
        EventType::NonpaddingReceived => Some(maybenot::TriggerEvent::NormalRecv),
        EventType::PaddingSent => Some(maybenot::TriggerEvent::PaddingSent {
            machine: machine_ids.get_machine_id(event.user_context)?.to_owned(),
        }),
    }
}

/// Handle for a set of DAITA machines.
/// Note: `close` is NOT called implicitly when this is dropped.
pub struct MachinistHandle {
    quit_event: talpid_windows::sync::Event,
}

impl MachinistHandle {
    fn new(quit_event: &talpid_windows::sync::Event) -> io::Result<MachinistHandle> {
        Ok(MachinistHandle {
            quit_event: quit_event.duplicate()?,
        })
    }

    /// Signal quit event
    pub fn close(&self) -> io::Result<()> {
        self.quit_event.set()
    }
}

pub struct Machinist {
    daita: Arc<Session>,
    machine_ids: MachineMap,
    machine_tasks: HashMap<usize, JoinHandle<()>>,
    tokio_handle: tokio::runtime::Handle,
    quit_event: talpid_windows::sync::Event,
    peer: PublicKey,
    mtu: u16,
}

// TODO: This is silly. Let me use the raw ID of MachineId, please.
struct MachineMap {
    id_to_num: HashMap<MachineId, usize>,
    num_to_id: HashMap<usize, MachineId>,
}

impl MachineMap {
    fn new() -> Self {
        Self {
            id_to_num: HashMap::new(),
            num_to_id: HashMap::new(),
        }
    }

    fn get_or_create_raw_id(&mut self, machine_id: MachineId) -> usize {
        *self.id_to_num.entry(machine_id).or_insert_with(|| {
            let raw_id = self.num_to_id.len();
            self.num_to_id.insert(raw_id, machine_id);
            raw_id
        })
    }

    fn get_machine_id(&self, raw_id: usize) -> Option<&MachineId> {
        self.num_to_id.get(&raw_id)
    }
}

impl Machinist {
    /// Spawn an actor that handles scheduling of Maybenot actions and forwards DAITA events to the
    /// framework.
    pub fn spawn(
        resource_dir: &Path,
        daita: Session,
        peer: PublicKey,
        mtu: u16,
    ) -> std::result::Result<MachinistHandle, Error> {
        const MAX_PADDING_BYTES: f64 = 0.0;
        const MAX_BLOCKING_BYTES: f64 = 0.0;

        static MAYBENOT_MACHINES: OnceCell<Vec<maybenot::Machine>> = OnceCell::new();

        let machines =
            MAYBENOT_MACHINES.get_or_try_init(|| load_maybenot_machines(resource_dir))?;

        let quit_event =
            talpid_windows::sync::Event::new(true, false).map_err(Error::InitializeQuitEvent)?;
        let handle = MachinistHandle::new(&quit_event).map_err(Error::InitializeHandle)?;

        let framework = maybenot::Framework::new(
            machines.clone(),
            MAX_PADDING_BYTES,
            MAX_BLOCKING_BYTES,
            std::time::Instant::now(),
            Rng::new(
                rand_chacha::ChaCha12Core::from_entropy(),
                RNG_RESEED_THRESHOLD,
                OsRng,
            ),
        )
        .map_err(|error| Error::InitializeMaybenot(error.to_string()))?;

        let daita = Arc::new(daita);
        let tokio_handle = tokio::runtime::Handle::current();

        std::thread::spawn(move || {
            Self {
                daita,
                machine_ids: MachineMap::new(),
                machine_tasks: HashMap::new(),
                tokio_handle,
                quit_event,
                peer,
                mtu,
            }
            .event_loop(framework);
        });

        Ok(handle)
    }

    fn event_loop(mut self, mut framework: maybenot::Framework<Vec<maybenot::Machine>, Rng>) {
        use windows_sys::Win32::Foundation::WAIT_OBJECT_0;

        loop {
            if unsafe { WaitForSingleObject(self.quit_event.as_raw(), 0) } == WAIT_OBJECT_0 {
                break;
            }

            let events = match self.wait_for_events() {
                Ok(events) => {
                    if events.is_empty() {
                        break;
                    }
                    events
                }
                Err(error) => {
                    log::error!("Error while waiting for DAITA events: {error}");
                    break;
                }
            };

            for action in framework.trigger_events(&events, std::time::Instant::now()) {
                self.handle_action(action);
            }
        }

        log::debug!("Stopped DAITA event loop");
    }

    fn handle_action(&mut self, action: &maybenot::action::TriggerAction) {
        match *action {
            maybenot::action::TriggerAction::Cancel { machine, timer } => {
                debug_assert_ne!(timer, Timer::Internal, "machine timers not implemented");

                let raw_id = self.machine_ids.get_or_create_raw_id(machine);

                // Drop all scheduled actions for a given machine
                if let Some(task) = self.machine_tasks.get_mut(&raw_id) {
                    task.abort();
                }
            }
            maybenot::action::TriggerAction::SendPadding {
                timeout,
                machine,
                replace,
                ..
            } => {
                let peer = self.peer.clone();

                let raw_id = self.machine_ids.get_or_create_raw_id(machine);
                self.machine_tasks.entry(raw_id).and_modify(|f| f.abort());

                let action = Action {
                    peer: *peer.as_bytes(),
                    action_type: ActionType::InjectPadding,
                    user_context: raw_id,
                    payload: ActionPayload {
                        padding: PaddingAction {
                            byte_count: self.mtu,
                            replace: if replace { 1 } else { 0 },
                        },
                    },
                };

                if timeout == Duration::ZERO {
                    if let Err(error) = self.daita.send_action(&action) {
                        log::error!("Failed to send DAITA action: {error}");
                    }
                } else {
                    // Schedule action on the tokio runtime
                    let daita = Arc::downgrade(&self.daita);
                    let task = self.tokio_handle.spawn(async move {
                        tokio::time::sleep(timeout).await;

                        let Some(daita) = daita.upgrade() else { return };

                        if let Err(error) = daita.send_action(&action) {
                            log::error!("Failed to send DAITA action: {error}");
                        }
                    });
                    self.machine_tasks.insert(raw_id, task);
                }
            }
            maybenot::action::TriggerAction::BlockOutgoing { .. } => {
                if cfg!(debug_assertions) {
                    unimplemented!("received BlockOutgoing action");
                }
            }
            maybenot::action::TriggerAction::UpdateTimer { .. } => {
                if cfg!(debug_assertions) {
                    unimplemented!("received UpdateTimer action");
                }
            }
        }
    }

    /// Take all events from the ring buffer while there are any left.
    /// If there are no events available, wait for events to arrive.
    /// Otherwise, break and return a non-zero number of events to be processed.
    /// If the quit event was signaled, this returns an empty vector.
    fn wait_for_events(&mut self) -> io::Result<Vec<maybenot::TriggerEvent>> {
        use windows_sys::Win32::Foundation::WAIT_OBJECT_0;

        let wait_events = [
            self.quit_event.as_raw(),
            self.daita.event_data_available_event() as isize,
        ];

        let mut event_buffer: [Event; EVENTS_CAPACITY] = unsafe { std::mem::zeroed() };

        loop {
            match self.daita.receive_events(&mut event_buffer) {
                Ok(events) => {
                    let converted_events: Vec<_> = events
                        .iter()
                        .filter(|event| &event.peer == self.peer.as_bytes())
                        .filter_map(|event| maybenot_event_from_event(event, &self.machine_ids))
                        .collect();
                    if !converted_events.is_empty() {
                        return Ok(converted_events);
                    }
                    // Try again if we only received events for irrelevant peers
                }
                Err(error) => {
                    if error.raw_os_error() == Some(ERROR_NO_MORE_ITEMS as i32) {
                        let wait_result = unsafe {
                            WaitForMultipleObjects(
                                u32::try_from(wait_events.len()).unwrap(),
                                wait_events.as_ptr(),
                                0,
                                INFINITE,
                            )
                        };

                        if wait_result == WAIT_OBJECT_0 {
                            // Quit event signaled
                            break Ok(vec![]);
                        }
                        if wait_result == WAIT_OBJECT_0 + 1 {
                            // Event object signaled -- try to receive more events
                            continue;
                        }
                    }
                    break Err(std::io::Error::last_os_error());
                }
            }
        }
    }
}

fn load_maybenot_machines(resource_dir: &Path) -> Result<Vec<maybenot::Machine>, Error> {
    let path = resource_dir.join("maybenot_machines");
    log::debug!("Reading maybenot machines from {}", path.display());

    let mut machines = vec![];
    let machines_str = fs::read_to_string(path).map_err(Error::EnumerateMachines)?;
    for machine_str in machines_str.lines() {
        let machine_str = machine_str.trim();
        if matches!(machine_str.chars().next(), None | Some('#')) {
            continue;
        }
        log::debug!("Adding maybenot machine: {machine_str}");
        machines.push(
            machine_str
                .parse::<maybenot::Machine>()
                .map_err(|_error| Error::InvalidMachine(machine_str.to_owned()))?,
        );
    }
    Ok(machines)
}

#[cfg(test)]
mod test {
    use super::load_maybenot_machines;
    use std::path::PathBuf;

    /// Test whether `maybenot_machines` in dist-assets contains valid machines.
    /// TODO: Remove when switching to dynamic machines.
    #[test]
    fn test_load_maybenot_machines() {
        let dist_assets = std::env::var("CARGO_MANIFEST_DIR")
            .map(PathBuf::from)
            .expect("CARGO_MANIFEST_DIR env var not set")
            .join("..")
            .join("dist-assets");

        load_maybenot_machines(&dist_assets).unwrap();
    }
}
