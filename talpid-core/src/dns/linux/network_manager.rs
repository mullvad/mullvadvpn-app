use dbus::{
    arg::{RefArg, Variant},
    stdintf::*,
    BusType,
};
use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader},
    net::IpAddr,
    path::Path,
};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(err_derive::Error, Debug)]
#[error(no_from)]
pub enum Error {
    #[error(display = "NetworkManager not detected")]
    NetworkManagerNotDetected(#[error(source)] dbus::Error),

    #[error(display = "NetworkManager is too old")]
    TooOldNetworkManager(#[error(source)] dbus::Error),

    #[error(display = "NetworkManager is not managing DNS")]
    NetworkManagerNotManagingDns,

    #[error(display = "Error while communicating over Dbus")]
    Dbus(#[error(source)] dbus::Error),

    #[error(display = "DNS is managed by systemd-resolved - NM can't enforce DNS globally")]
    SystemdResolved,
}

const NM_BUS: &str = "org.freedesktop.NetworkManager";
const NM_TOP_OBJECT: &str = "org.freedesktop.NetworkManager";
const NM_DNS_MANAGER: &str = "org.freedesktop.NetworkManager.DnsManager";
const NM_DNS_MANAGER_PATH: &str = "/org/freedesktop/NetworkManager/DnsManager";
const NM_OBJECT_PATH: &str = "/org/freedesktop/NetworkManager";
const RPC_TIMEOUT_MS: i32 = 3000;
const GLOBAL_DNS_CONF_KEY: &str = "GlobalDnsConfiguration";
const RC_MANAGEMENT_MODE_KEY: &str = "RcManager";
const DNS_MODE_KEY: &str = "Mode";

pub struct NetworkManager {
    dbus_connection: dbus::Connection,
}


impl NetworkManager {
    pub fn new() -> Result<Self> {
        let dbus_connection =
            dbus::Connection::get_private(BusType::System).map_err(Error::Dbus)?;
        let manager = NetworkManager { dbus_connection };
        manager.ensure_resolv_conf_is_managed()?;
        manager.ensure_network_manager_exists()?;
        Ok(manager)
    }

    fn ensure_network_manager_exists(&self) -> Result<()> {
        let _: Box<dyn RefArg> = self
            .as_manager()
            .get(&NM_TOP_OBJECT, GLOBAL_DNS_CONF_KEY)
            .map_err(Error::NetworkManagerNotDetected)?;
        Ok(())
    }

    fn ensure_resolv_conf_is_managed(&self) -> Result<()> {
        // check if NM is set to manage resolv.conf
        let management_mode: String = self
            .dbus_connection
            .with_path(NM_BUS, NM_DNS_MANAGER_PATH, RPC_TIMEOUT_MS)
            .get(NM_DNS_MANAGER, RC_MANAGEMENT_MODE_KEY)
            .map_err(Error::TooOldNetworkManager)?;
        if management_mode == "unmanaged" {
            return Err(Error::NetworkManagerNotManagingDns);
        }

        let dns_mode: String = self
            .dbus_connection
            .with_path(NM_BUS, NM_DNS_MANAGER_PATH, RPC_TIMEOUT_MS)
            .get(NM_DNS_MANAGER, DNS_MODE_KEY)
            .map_err(Error::Dbus)?;

        match dns_mode.as_ref() {
            // NetworkManager can only set DNS globally if it's not managing DNS through
            // systemd-resolved.
            "systemd-resolved" => return Err(Error::SystemdResolved),
            // If NetworkManager isn't managing DNS for us, it's useless.
            "none" => return Err(Error::NetworkManagerNotManagingDns),
            _ => (),
        };


        let expected_resolv_conf = "/var/run/NetworkManager/resolv.conf";
        let actual_resolv_conf = "/etc/resolv.conf";
        if !eq_file_content(&expected_resolv_conf, &actual_resolv_conf) {
            log::debug!("/etc/resolv.conf differs from reference resolv.conf, therefore NM is not managing DNS");
            return Err(Error::NetworkManagerNotManagingDns);
        }

        Ok(())
    }

    fn as_manager(&self) -> dbus::ConnPath<'_, &dbus::Connection> {
        self.dbus_connection
            .with_path(NM_BUS, NM_OBJECT_PATH, RPC_TIMEOUT_MS)
    }

    pub fn set_dns(&mut self, servers: &[IpAddr]) -> Result<()> {
        self.set_global_dns(create_global_settings(servers))
    }

    fn set_global_dns(&mut self, config: GlobalDnsConfig) -> Result<()> {
        self.as_manager()
            .set(NM_TOP_OBJECT, GLOBAL_DNS_CONF_KEY, config)
            .map_err(Error::Dbus)
    }

    pub fn reset(&mut self) -> Result<()> {
        self.set_global_dns(create_empty_global_settings())
    }
}

type GlobalDnsConfig = HashMap<&'static str, Variant<Box<dyn RefArg>>>;

// The NetworkManager GlobalDnsConfiguration schema looks something like this
// {
//  "searches": ["example.com", "search-domain.com"],
//  "options": "this field is currently unused",
//  "domains": {
//   "*": {
//     "servers": [ "1.1.1.1" ]
//   }
//   "example.com": {
//     "servers": [ "8.8.8.8", "8.8.4.4" ]
//   }
//  }
// }
fn create_global_settings(server_list: &[IpAddr]) -> GlobalDnsConfig {
    let mut global_settings = HashMap::new();
    let mut domain_settings = HashMap::new();
    let mut specific_domain_config = HashMap::new();

    let dns_server_list = as_variant(
        server_list
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>(),
    );
    specific_domain_config.insert("servers".to_owned(), dns_server_list);
    domain_settings.insert("*".to_owned(), as_variant(specific_domain_config));
    global_settings.insert("domains", as_variant(domain_settings));
    global_settings.insert("searches", as_variant(vec![] as Vec<String>));
    global_settings.insert("options", as_variant(vec![] as Vec<String>));

    global_settings
}

fn create_empty_global_settings() -> GlobalDnsConfig {
    HashMap::new()
}

fn as_variant<T: RefArg + 'static>(t: T) -> Variant<Box<dyn RefArg>> {
    Variant(Box::new(t) as Box<dyn RefArg>)
}

fn eq_file_content<P: AsRef<Path>>(a: &P, b: &P) -> bool {
    let file_a = match File::open(a).map(BufReader::new) {
        Ok(file) => file,
        Err(e) => {
            log::debug!("Failed top open file {}: {}", a.as_ref().display(), e);
            return false;
        }
    };
    let file_b = match File::open(b).map(BufReader::new) {
        Ok(file) => file,
        Err(e) => {
            log::debug!("Failed top open file {}: {}", b.as_ref().display(), e);
            return false;
        }
    };

    file_a
        .lines()
        .zip(file_b.lines())
        .all(|(a, b)| match (a, b) {
            (Ok(a), Ok(b)) => a == b,
            _ => false,
        })
}
