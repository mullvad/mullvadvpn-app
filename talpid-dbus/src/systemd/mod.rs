use dbus::{
    blocking::{stdintf::org_freedesktop_dbus::Properties, Proxy, SyncConnection},
    message::MatchRule,
    Path,
};
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};

mod unit;
use unit::Unit;

type Result<T> = std::result::Result<T, Error>;

#[derive(err_derive::Error, Debug)]
#[error(no_from)]
pub enum Error {
    #[error(display = "Failed to create a DBus connection")]
    ConnectError(#[error(source)] dbus::Error),

    #[error(display = "ListUnitsByName failed")]
    ListError(#[error(source)] dbus::Error),

    #[error(display = "Failed to read UnitFileState property")]
    ReadUnitFileStateError(#[error(source)] dbus::Error),

    #[error(display = "Failed to add matcher")]
    MatcherError(#[error(source)] dbus::Error),

    #[error(display = "Failed to subscribe")]
    Subscribe(#[error(source)] dbus::Error),

    #[error(display = "Failed to unsubscribe")]
    Unsubscribe(#[error(source)] dbus::Error),

    #[error(display = "Unit doesn't exist")]
    UnitDoesntExist,
}

const SYSTEMD_BUS: &str = "org.freedesktop.systemd1";
const SYSTEMD_PATH: &str = "/org/freedesktop/systemd1";
const MANAGER_INTERFACE: &str = "org.freedesktop.systemd1.Manager";
const UNIT_INTERFACE: &str = "org.freedesktop.systemd1.Unit";
const LIST_UNITS_BY_NAMES_METHOD: &str = "ListUnitsByNames";
const JOB_REMOVED_SIGNAL: &str = "JobRemoved";
const SUBSCRIBE: &str = "Subscribe";
const UNSUBSCRIBE: &str = "Unsubscribe";

pub const SYSTEMD_RESOLVED_SERVICE: &str = "systemd-resolved.service";
pub const NETWORK_MANAGER_SERVICE: &str = "NetworkManager.service";
const RPC_TIMEOUT: Duration = Duration::from_secs(1);
const ACTIVE_CHECK_SLEEP_INTERVAL: Duration = Duration::from_secs(30);
#[derive(Clone)]
pub struct Systemd {
    pub dbus_connection: Arc<SyncConnection>,
}

impl Systemd {
    pub fn new() -> Result<Self> {
        let sd = Self {
            dbus_connection: crate::get_connection().map_err(Error::ConnectError)?,
        };
        sd.subscribe()?;
        Ok(sd)
    }

    /// Calls the `Subscribe` RPC to register with systemd - otherwise systemd won't send signals
    fn subscribe(&self) -> Result<()> {
        self.as_manager_object()
            .method_call(MANAGER_INTERFACE, SUBSCRIBE, ())
            .map_err(Error::Subscribe)
    }

    fn unsubscribe(&self) -> Result<()> {
        self.as_manager_object()
            .method_call(MANAGER_INTERFACE, UNSUBSCRIBE, ())
            .map_err(Error::Unsubscribe)
    }

    pub fn wait_for_network_manager_to_be_active(&self) -> Result<bool> {
        let unit = self
            .get_unit(NETWORK_MANAGER_SERVICE)?
            .ok_or(Error::UnitDoesntExist)?;
        if unit.is_active() {
            return Ok(true);
        }
        self.wait_for_unit_to_be_active(&unit, ACTIVE_CHECK_SLEEP_INTERVAL)
    }

    pub fn wait_for_systemd_resolved_to_be_active(&self) -> Result<bool> {
        let unit = self
            .get_unit(SYSTEMD_RESOLVED_SERVICE)?
            .ok_or(Error::UnitDoesntExist)?;
        self.wait_for_unit_to_be_active(&unit, ACTIVE_CHECK_SLEEP_INTERVAL)
    }

    fn wait_for_unit_to_be_active(&self, unit: &Unit, timeout: Duration) -> Result<bool> {
        if unit.is_active() {
            return Ok(true);
        }
        self.wait_for_unit_to_be_active_inner(unit, timeout)
    }

    /// Returns false if the unit didn't start before hitting the timeout, otherwise returns true.
    pub fn wait_for_unit_to_be_active_inner(&self, unit: &Unit, timeout: Duration) -> Result<bool> {
        let removed_match_rule = MatchRule::new_signal(MANAGER_INTERFACE, JOB_REMOVED_SIGNAL);
        let unit_not_running = Arc::new(AtomicBool::new(true));
        let cb_unit_not_running = unit_not_running.clone();
        let unit_name = unit.name.clone();

        let conn = self.clone();
        let remove_matcher = self
            .dbus_connection
            .add_match(
                removed_match_rule,
                move |job_finished: JobRemovedSignal, _connection, _message| {
                    if job_finished.primary_name == unit_name {
                        println!(
                            "Actually matched a job of a unit - {:?} with '{}'",
                            job_finished, unit_name
                        );
                        match conn.get_unit(&unit_name) {
                            Ok(Some(unit)) => {
                                let not_running_now = !unit.is_active();
                                cb_unit_not_running.store(not_running_now, Ordering::Release);
                                log::error!("Unit {unit_name} is not running: {not_running_now}");
                            }
                            Ok(None) => {
                                log::error!("Unit no longer exists");
                            }
                            Err(err) => {
                                log::error!(
                                    "Failed to get unit data after waiting on it's job: {}",
                                    err
                                );
                            }
                        }
                    }
                    true
                },
            )
            .map_err(Error::MatcherError)?;

        let deadline = Instant::now() + timeout;
        while Instant::now() < deadline && unit_not_running.load(Ordering::Acquire) {
            self.dbus_connection
                .process(RPC_TIMEOUT)
                .map_err(Error::MatcherError)?;
        }

        self.dbus_connection
            .remove_match(remove_matcher)
            .map_err(Error::MatcherError)?;

        Ok(!unit_not_running.load(Ordering::Acquire)
            || self
                .get_unit(&unit.name)
                .ok()
                .flatten()
                .as_ref()
                .map(Unit::is_active)
                .map(|is_active| {
                    println!("Only checking if unit is active after not receiving a signal about it - {is_active}");
                    is_active
                })
                .unwrap_or(false))
    }

    pub fn systemd_resolved_will_run(&self) -> Result<bool> {
        self.unit_will_run(SYSTEMD_RESOLVED_SERVICE)
    }

    pub fn network_manager_will_run(&self) -> Result<bool> {
        self.unit_will_run(NETWORK_MANAGER_SERVICE)
    }

    pub fn network_manager_is_active(&self) -> Result<bool> {
        match self.get_unit(NETWORK_MANAGER_SERVICE)? {
            Some(unit) => Ok(unit.is_active()),
            None => Ok(false),
        }
    }

    fn unit_will_run(&self, unit_name: &str) -> Result<bool> {
        let unit = match self.get_unit(unit_name)? {
            Some(unit) => unit,
            None => return Ok(false),
        };

        if unit.is_active() {
            return Ok(true);
        }

        self.unit_is_enabled(&unit)
    }

    fn unit_is_enabled(&self, unit: &Unit) -> Result<bool> {
        Ok(self.unit_file_state(&unit.object_path)? == "enabled")
    }

    pub fn unit_is_running(&self, unit: &str) -> Result<bool> {
        let unit = self.get_unit(unit)?;
        Ok(unit.as_ref().map(Unit::is_active).unwrap_or(false))
    }

    pub fn get_unit(&self, unit: &str) -> Result<Option<Unit>> {
        let (mut units,) = self.list_units(&[unit])?;
        if units.len() > 1 {
            log::debug!("Unexpected length of units returend - {}", units.len());
        }
        Ok(units.pop())
    }

    fn as_manager_object(&self) -> Proxy<'_, &SyncConnection> {
        Proxy::new(
            SYSTEMD_BUS,
            SYSTEMD_PATH,
            RPC_TIMEOUT,
            &self.dbus_connection,
        )
    }

    fn as_unit<'a>(&'a self, unit_path: &'a Path<'_>) -> Proxy<'a, &SyncConnection> {
        Proxy::new(SYSTEMD_BUS, unit_path, RPC_TIMEOUT, &self.dbus_connection)
    }

    fn unit_file_state(&self, unit_path: &Path<'_>) -> Result<String> {
        self.as_unit(unit_path)
            .get(UNIT_INTERFACE, "UnitFileState")
            .map_err(Error::ReadUnitFileStateError)
    }

    fn list_units(&self, units_by_name: &[&str]) -> Result<(Vec<Unit>,)> {
        self.as_manager_object()
            .method_call(
                MANAGER_INTERFACE,
                LIST_UNITS_BY_NAMES_METHOD,
                (units_by_name,),
            )
            .map_err(Error::ListError)
    }
}

impl Drop for Systemd {
    fn drop(&mut self) {
        if let Err(err) = self.unsubscribe() {
            log::error!("Faled to unsubscribe when dropping systemd connection: {err}");
        }
    }
}

#[derive(Debug)]
struct JobRemovedSignal {
    pub _job_id: u32,
    pub _job_path: Path<'static>,
    pub primary_name: String,
    pub _job_result: String,
}

impl dbus::message::SignalArgs for JobRemovedSignal {
    const NAME: &'static str = MANAGER_INTERFACE;
    const INTERFACE: &'static str = JOB_REMOVED_SIGNAL;
}

impl dbus::arg::ReadAll for JobRemovedSignal {
    fn read(i: &mut dbus::arg::Iter) -> std::result::Result<Self, dbus::arg::TypeMismatchError> {
        Ok(JobRemovedSignal {
            _job_id: i.read()?,
            _job_path: i.read()?,
            primary_name: i.read()?,
            _job_result: i.read()?,
        })
    }
}
