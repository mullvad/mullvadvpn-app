use super::WIREGUARD_KEY_LENGTH;
use maybenot::framework::MachineId;
use once_cell::sync::OnceCell;
use std::{collections::HashMap, fs, io, path::Path, sync::Arc, time::Duration};
use talpid_types::net::wireguard::PublicKey;
use tokio::task::JoinHandle;

// TODO: This was an i32 for wireguard-nt, ask David about it
type BOOLEAN = bool;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Failed to find maybenot machines
    #[error("Failed to enumerate maybenot machines")]
    EnumerateMachines(#[source] io::Error),
    /// Failed to parse maybenot machine
    #[error("Failed to parse maybenot machine \"{}\"", 0)]
    InvalidMachine(String),

    /// Failed to initialize maybenot framework
    #[error("Failed to initialize maybenot framework: {}", 0)]
    InitializeMaybenot(String),
}

// See DAITA_EVENT_TYPE:
// https://github.com/mullvad/wireguard-nt-priv/blob/mullvad-patches/driver/daita.h
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

// See DAITA_EVENT:
// https://github.com/mullvad/wireguard-nt-priv/blob/mullvad-patches/driver/daita.h
#[repr(C)]
#[derive(Debug, Default)]
pub struct Event {
    pub peer: [u8; WIREGUARD_KEY_LENGTH],
    pub event_type: EventType,
    pub xmit_bytes: u16,
    pub user_context: usize,
}

// See DAITA_ACTION_TYPE:
// https://github.com/mullvad/wireguard-nt-priv/blob/mullvad-patches/driver/daita.h
#[repr(C)]
#[derive(Debug)]
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
const EVENTS_CAPACITY: u32 = 1000;
/// Maximum number of actions that can be stored in the underlying buffer
const ACTIONS_CAPACITY: u32 = 1000;

#[derive(Debug)]
pub struct Session {
    tunnel_handle: i32,
}

impl Session {
    /// Call `wgActivateDaita` for an existing WireGuard interface
    pub(super) fn from_adapter(tunnel_handle: i32) -> io::Result<Session> {
        let res =
            unsafe { super::wgActivateDaita(tunnel_handle, EVENTS_CAPACITY, ACTIONS_CAPACITY) };
        if !res {
            // TODO: return error
            panic!("Failed to activate DAITA")
        }
        Ok(Self { tunnel_handle })
    }

    pub fn receive_events(&self) -> super::Result<Event> {
        let mut buffer = Event::default();
        let res = unsafe { super::wgReceiveEvent(self.tunnel_handle, &mut buffer) };
        if res == 0 {
            Ok(buffer)
        } else {
            Err(crate::TunnelError::DaitaReceiveEvent(res))
        }
    }

    pub fn send_action(&self, action: &Action) -> io::Result<()> {
        let res = unsafe { super::wgSendAction(self.tunnel_handle, action) };
        if !res == 0 {
            // TODO: return error
            panic!("Failed to send DAITA action, error code {res}")
        }
        log::info!("Send DAITA action {:?}", action.action_type);
        Ok(())
    }
}

fn maybenot_event_from_event(
    event: &Event,
    machine_ids: &MachineMap,
    override_size: Option<u16>,
) -> Option<maybenot::framework::TriggerEvent> {
    let xmit_bytes = override_size.unwrap_or(event.xmit_bytes);
    match event.event_type {
        EventType::PaddingReceived => Some(maybenot::framework::TriggerEvent::PaddingRecv {
            bytes_recv: xmit_bytes,
        }),
        EventType::NonpaddingSent => Some(maybenot::framework::TriggerEvent::NonPaddingSent {
            bytes_sent: xmit_bytes,
        }),
        EventType::NonpaddingReceived => Some(maybenot::framework::TriggerEvent::NonPaddingRecv {
            bytes_recv: xmit_bytes,
        }),
        EventType::PaddingSent => Some(maybenot::framework::TriggerEvent::PaddingSent {
            bytes_sent: xmit_bytes,
            machine: machine_ids.get_machine_id(event.user_context)?.to_owned(),
        }),
    }
}

pub type MachinistHandle = std::thread::JoinHandle<()>;

pub struct Machinist {
    daita: Arc<Session>,
    machine_ids: MachineMap,
    machine_tasks: HashMap<usize, JoinHandle<()>>,
    tokio_handle: tokio::runtime::Handle,
    peer: PublicKey,
    override_size: Option<u16>,
}

// TODO(sebastian): Remove this and use `MachineId` directly in the hashmap?
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

        static MAYBENOT_MACHINES: OnceCell<Vec<maybenot::machine::Machine>> = OnceCell::new();

        let machines = MAYBENOT_MACHINES.get_or_try_init(|| {
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
                        .parse::<maybenot::machine::Machine>()
                        .map_err(|_error| Error::InvalidMachine(machine_str.to_owned()))?,
                );
            }
            Ok(machines)
        })?;

        let framework = maybenot::framework::Framework::new(
            machines.clone(),
            MAX_PADDING_BYTES,
            MAX_BLOCKING_BYTES,
            mtu,
            std::time::Instant::now(),
        )
        .map_err(|error| Error::InitializeMaybenot(error.to_string()))?;

        let daita = Arc::new(daita);
        let tokio_handle = tokio::runtime::Handle::current();

        Ok(std::thread::spawn(move || {
            Self {
                daita,
                machine_ids: MachineMap::new(),
                machine_tasks: HashMap::new(),
                tokio_handle,
                peer,
                // TODO: We're assuming that constant packet size is always enabled here
                override_size: Some(mtu),
            }
            .event_loop(framework);
        }))
    }

    fn event_loop(
        mut self,
        mut framework: maybenot::framework::Framework<Vec<maybenot::machine::Machine>>,
    ) {
        loop {
            let event = match self.wait_for_events() {
                Ok(event) => event,
                Err(error) => {
                    log::error!("Error while waiting for DAITA events: {error}");
                    break;
                }
            };

            for action in framework.trigger_events(&[event], std::time::Instant::now()) {
                self.handle_action(action);
            }
        }

        log::debug!("Stopped DAITA event loop");
    }

    fn handle_action(&mut self, action: &maybenot::framework::Action) {
        match *action {
            maybenot::framework::Action::Cancel { machine } => {
                let raw_id = self.machine_ids.get_or_create_raw_id(machine);

                // Drop all scheduled actions for a given machine
                if let Some(task) = self.machine_tasks.get_mut(&raw_id) {
                    task.abort();
                }
            }
            maybenot::framework::Action::InjectPadding {
                timeout,
                size,
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
                            byte_count: size,
                            replace,
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
            maybenot::framework::Action::BlockOutgoing { .. } => {}
        }
    }

    fn wait_for_events(&mut self) -> super::Result<maybenot::framework::TriggerEvent> {
        loop {
            let event = self.daita.receive_events()?;
            if &event.peer == self.peer.as_bytes() {
                if let Some(event) =
                    maybenot_event_from_event(&event, &self.machine_ids, self.override_size)
                {
                    return Ok(event);
                }
            }
        }
    }
}
