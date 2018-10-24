extern crate dbus;

use std::collections::HashMap;
use std::net::IpAddr;

use self::dbus::arg::{RefArg, Variant};
use self::dbus::stdintf::*;
use self::dbus::BusType;

error_chain! {
    errors {
        NoNetworkManager {
            description("NetworkManager not detected")
        }
    }

    foreign_links {
        DbusError(dbus::Error);
    }
}

const NM_BUS: &str = "org.freedesktop.NetworkManager";
const NM_TOP_OBJECT: &str = "org.freedesktop.NetworkManager";
const NM_OBJECT_PATH: &str = "/org/freedesktop/NetworkManager";
const RPC_TIMEOUT_MS: i32 = 1000;
const GLOBAL_DNS_CONF_KEY: &str = "GlobalDnsConfiguration";

pub struct NetworkManager {
    dbus_connection: dbus::Connection,
}


impl NetworkManager {
    pub fn new() -> Result<Self> {
        let dbus_connection = dbus::Connection::get_private(BusType::System)?;
        let manager = NetworkManager { dbus_connection };
        manager.ensure_network_manager_exists()?;
        Ok(manager)
    }

    fn ensure_network_manager_exists(&self) -> Result<()> {
        let _: Box<RefArg> = self
            .as_manager()
            .get(&NM_TOP_OBJECT, GLOBAL_DNS_CONF_KEY)
            .chain_err(|| ErrorKind::NoNetworkManager)?;
        Ok(())
    }

    fn as_manager(&self) -> dbus::ConnPath<&dbus::Connection> {
        self.dbus_connection
            .with_path(NM_BUS, NM_OBJECT_PATH, RPC_TIMEOUT_MS)
    }

    pub fn set_dns(&mut self, servers: &[IpAddr]) -> Result<()> {
        self.set_global_dns(create_global_settings(servers))
    }

    fn set_global_dns(&mut self, config: GlobalDnsConfig) -> Result<()> {
        self.as_manager()
            .set(NM_TOP_OBJECT, GLOBAL_DNS_CONF_KEY, config)
            .map_err(|e| e.into())
    }

    pub fn reset(&mut self) -> Result<()> {
        self.set_global_dns(create_empty_global_settings())
    }
}

type GlobalDnsConfig = HashMap<&'static str, Variant<Box<RefArg>>>;

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

fn as_variant<T: RefArg + 'static>(t: T) -> Variant<Box<RefArg>> {
    Variant(Box::new(t) as Box<RefArg>)
}
