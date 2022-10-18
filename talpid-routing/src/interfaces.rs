use std::{collections::BTreeMap, net::IpAddr, time::Duration};

use system_configuration::{
    core_foundation::string::CFString,
    network_configuration::{SCNetworkService, SCNetworkSet},
    preferences::SCPreferences,
};
use talpid_time::Instant;

use super::{
    ip6addr_ext::IpAddrExt,
    watch::data::{self, AddressMessage, RouteDestination},
    Error, Result,
};

const NON_DEFAULT_ROUTE_VALIDITY_TIMEOUT: Duration = Duration::from_secs(120);

/// Keepes track of valid interfaces
pub struct Interfaces {
    map: BTreeMap<u16, Interface>,
    current_v4_interface: Option<u16>,
    current_v6_interface: Option<u16>,
}

impl Interfaces {
    pub fn new() -> Interfaces {
        Self {
            map: BTreeMap::new(),
            current_v4_interface: None,
            current_v6_interface: None,
        }
    }

    pub fn confirm_route(&mut self, interface: u16, destination: &RouteDestination) {
        if let Some(iface) = self.map.get_mut(&interface) {
            iface.confirm_route(destination);
        }
    }

    // Currently only lookgs for best V4 default interface
    pub fn get_best_default_interface_v4(&self) -> Result<Option<BestRoute>> {
        self.get_best_interface(&|interface| interface.best_v4_route())
    }

    // Currently only lookgs for best V6 default interface
    pub fn get_best_default_interface_v6(&self) -> Result<Option<BestRoute>> {
        self.get_best_interface(&|interface| interface.best_v6_route())
    }

    fn get_best_interface(
        &self,
        ipv_fn: &dyn Fn(&Interface) -> Option<BestRoute>,
    ) -> Result<Option<BestRoute>> {
        let mut ordered_interfaces = order_of_interfaces()
            .into_iter()
            .filter_map(|iface_name| {
                self.map.iter().find_map(|(idx, interface)| {
                    if interface.name == iface_name {
                        Some(*idx)
                    } else {
                        None
                    }
                })
            })
            .filter_map(|index| self.map.get(&index))
            .filter_map(|interface| Some((interface, ipv_fn(interface)?)))
            .collect::<Vec<_>>();

        if ordered_interfaces.is_empty() {
            log::error!("Failed to obtain a list of valid network services");
            return Ok(None);
        }

        ordered_interfaces.sort_by_key(|(_interface, best_route)| best_route.validity);
        Ok(ordered_interfaces
            .into_iter()
            .next()
            .map(|(_, best_route)| best_route))
    }

    pub fn handle_add_address(&mut self, address: AddressMessage) -> bool {
        let interface = match self.map.get_mut(&address.index()) {
            Some(interface) => interface,
            None => {
                log::error!(
                    "Received address message for non-existant interface with index {}",
                    address.index()
                );
                return false;
            }
        };

        match address.address() {
            Ok(addr) => interface.add_address(addr),
            Err(err) => {
                log::error!("Failed to get interface address from address message: {err:?}");
                false
            }
        }
    }

    pub fn handle_delete_address(&mut self, address: AddressMessage) -> bool {
        let interface = match self.map.get_mut(&address.index()) {
            Some(interface) => interface,
            None => {
                log::error!(
                    "Received address message with an unknown interface {}",
                    address.index()
                );
                return false;
            }
        };

        match address.address() {
            Ok(addr) => interface.addresses.remove(&addr).is_some(),
            Err(err) => {
                log::error!("Failed to get interface address from address message: {err:?}");
                false
            }
        }
    }

    // returning true implies that the best default interface might've changed
    pub fn handle_iface_msg(&mut self, interface: data::Interface) -> Result<bool> {
        let index = interface.index();
        if interface.is_up() {
            if self.map.contains_key(&index) {
                return Ok(false);
            }
            self.map.insert(index, Interface::new(index)?);
            // just because an interface is added doesn't imply that routes will change - have to
            // wait for new addresses and routes to come in.
            Ok(false)
        } else {
            Ok(self.map.remove(&index).is_some())
        }
    }

    pub fn handle_add_route(&mut self, route: &data::RouteMessage) -> Result<bool> {
        let destination = route.destination_ip().map_err(Error::InvalidData)?;

        let mut new_v4_route = false;
        let mut new_v6_route = false;

        if route.is_ipv4() {
            match (route.ifscope(), route.interface_sockaddr_index()) {
                (Some(index), _) | (_, Some(index)) => match self.map.get_mut(&index) {
                    Some(interface) => {
                        let destination =
                            RouteDestination::try_from(route).map_err(Error::InvalidData)?;
                        interface.add_route(destination);
                        if route.is_ipv4() {
                            new_v4_route = Some(index) != self.current_v4_interface;
                        } else {
                            new_v6_route = Some(index) != self.current_v6_interface;
                        }
                    }
                    None => {
                        log::error!("Received a route with destination {:?} about through an unknown interface {index}", route.destination_ip());
                    }
                },
                _ => (),
            }
        }

        if new_v4_route {
            let new_interface = self
                .get_best_default_interface_v4()?
                .map(|route| route.iface_index);
            if new_interface != self.current_v4_interface {
                self.current_v4_interface = new_interface;
                return Ok(true);
            }
        }

        if new_v6_route {
            let new_interface = self
                .get_best_default_interface_v6()?
                .map(|route| route.iface_index);
            if new_interface != self.current_v6_interface {
                self.current_v6_interface = new_interface;
                return Ok(true);
            }
        }

        Ok(false)
    }

    pub fn handle_delete_route(&mut self, route: &data::RouteMessage) -> Result<bool> {
        if let Some(ifscope) = route.ifscope() {
            if let Some(interface) = self.map.get_mut(&ifscope) {
                let destination = route.try_into().map_err(Error::RouteDestination)?;
                interface.remove_route(&destination);
                return Ok(true);
            } else {
                log::error!("Received route message about unknown interface");
                return Ok(false);
            }
        }

        let iface_addr = match route.interface_address() {
            Some(addr) => addr,
            None => {
                return Ok(false);
            }
        };

        let interface = self
            .map
            .values_mut()
            .find(|interface| interface.has_addr(&iface_addr));

        if let Some(interface) = interface {
            let destination = route.try_into().map_err(Error::RouteDestination)?;
            interface.remove_route(&destination);
            return Ok(true);
        }

        Ok(false)
    }

    pub fn handle_changed_route(&mut self, route: &data::RouteMessage) -> Result<bool> {
        // If an ifscoped route is changed, it can be interpreted as though a new route has been
        // added, if the old is removed first.
        if route.is_ifscope() {
            return Ok(false);
        }

        Ok(self.handle_delete_route(route)? || self.handle_add_route(route)?)
    }
}

/// Represents all the data about the current best route to the internet
pub struct BestRoute {
    iface_index: u16,
    destination: RouteDestination,
    validity: RouteValidity,
}

impl BestRoute {
    /// Returns amount of time between now and until the route will no longer be considered valid.
    pub fn timeout(&self) -> Option<Instant> {
        None
        // match self.validity {
        //     RouteValidity::Unknown()
        // }
    }
}

pub struct InterfaceIdentifier {
    index: u16,
    name: String,
}

pub struct Interface {
    /// Network interface index
    index: u16,
    /// BSD name of the network interface
    name: String,
    /// routes assigned to interface, should not be used to track ifscoped routes
    routes: BTreeMap<RouteDestination, RouteValidity>,
    /// Addresses assigned to the network interface
    addresses: BTreeMap<IpAddr, Instant>,
}

impl Interface {
    fn new(index: u16) -> Result<Self> {
        let interfaces = nix::net::if_::if_nameindex().map_err(Error::GetInterfaceNames)?;
        let c_name = interfaces
            .iter()
            .find(|iface| iface.index() == index.into())
            .map(|iface| iface.name())
            .ok_or(Error::GetInterfaceName)?;

        let name = match c_name.to_str() {
            Ok(name) => name.to_owned(),
            Err(_) => {
                log::error!("Interface name is not valid UTF-8: {:?}", c_name);
                return Err(Error::GetInterfaceName);
            }
        };

        Ok(Self {
            name,
            index,
            routes: Default::default(),
            addresses: Default::default(),
        })
    }

    fn best_v4_route(&self) -> Option<BestRoute> {
        let mut candidates = self
            .routes
            .iter()
            .filter(|(destination, validity)| destination.is_ipv4() && validity.is_valid())
            .collect::<Vec<_>>();

        candidates.first().map(|(destination, validity)| BestRoute {
            iface_index: self.index,
            destination: (*destination).clone(),
            validity: **validity,
        })
    }

    fn best_v6_route(&self) -> Option<BestRoute> {
        let mut candidates = self
            .routes
            .iter()
            .filter(|(destination, validity)| !destination.is_ipv4() && validity.is_valid())
            .collect::<Vec<_>>();

        candidates.sort_by_key(|(destination, validity)| *validity);

        candidates.first().map(|(destination, validity)| BestRoute {
            iface_index: self.index,
            destination: (*destination).clone(),
            validity: **validity,
        })
    }

    fn add_route(&mut self, destination: RouteDestination) -> bool {
        let validity = if destination.is_default() {
            RouteValidity::Default
        } else {
            RouteValidity::Unknown(Instant::now())
        };
        self.routes.insert(destination, validity).is_some()
    }

    fn confirm_route(&mut self, destination: &RouteDestination) {
        if let Some(route) = self.routes.get_mut(&destination) {
            *route = RouteValidity::Confirmed;
        }
    }

    fn remove_route(&mut self, destination: &RouteDestination) {
        self.routes.remove(destination);
    }

    fn has_v4_default_route(&self) -> bool {
        self.routes
            .keys()
            .any(|route| route.is_ipv4() && route.is_default())
    }

    fn has_v6_default_route(&self) -> bool {
        self.routes
            .keys()
            .any(|route| !route.is_ipv4() && route.is_default())
    }

    fn add_address(&mut self, address: IpAddr) -> bool {
        self.addresses.insert(address, Instant::now()).is_some()
    }

    fn has_addr(&self, iface_addr: &IpAddr) -> bool {
        self.addresses.contains_key(iface_addr)
    }
}

fn order_of_interfaces() -> Vec<String> {
    let prefs = SCPreferences::default(&CFString::new("talpid-routing"));
    let services = SCNetworkService::get_services(&prefs);
    let set = SCNetworkSet::new(&prefs);
    let service_order = set.service_order();

    service_order
        .iter()
        .filter_map(|service_id| {
            services
                .iter()
                .find(|service| service.id().as_ref() == Some(&*service_id))
                .and_then(|service| service.network_interface()?.bsd_name())
                .map(|cf_name| cf_name.to_string())
        })
        .collect::<Vec<_>>()
}

/// Represents whether a route is considered to provide internet.
#[derive(Clone, Copy)]
pub enum RouteValidity {
    /// Route is default, hence it's expected it must provide connectivity.
    Default,
    /// If a non-default route seems to provide connectivity, then this should be considered to be
    /// a valid route.
    Confirmed,
    /// When a new route appears that isn't a default one, we can assume it's interface may provide
    /// internet connectivity. In this case, it should be valid for a specific amount of time -
    /// afterwards, if connectivity cannot be established, it it can be assumed it doesn't provide
    /// connectivity.
    Unknown(Instant),
}

impl RouteValidity {
    fn is_valid(&self) -> bool {
        match self {
            Self::Default | Self::Confirmed => true,
            Self::Unknown(time_seen) => {
                time_seen.duration_since(Instant::now()) <= NON_DEFAULT_ROUTE_VALIDITY_TIMEOUT
            }
        }
    }
}

impl std::fmt::Debug for RouteValidity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RouteValidity::")?;
        match self {
            Self::Default => write!(f, "Default"),
            Self::Confirmed => write!(f, "Confirmed"),
            Self::Unknown(time_seen) => {
                if self.is_valid() {
                    let msecs_valid = time_seen.duration_since(Instant::now()).as_millis();
                    write!(f, "Uknown(valid for {msecs_valid} ms)")
                } else {
                    write!(f, "Uknown(expired)")
                }
            }
        }
    }
}

impl PartialEq for RouteValidity {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Default, Self::Default) => true,
            (Self::Confirmed, Self::Confirmed) => true,
            (Self::Unknown(time_created), Self::Unknown(other)) => {
                let now = Instant::now();
                time_created.duration_since(now) == other.duration_since(now)
            }
            _ => false,
        }
    }
}
impl Eq for RouteValidity {}

impl PartialOrd for RouteValidity {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for RouteValidity {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self == other {
            return std::cmp::Ordering::Equal;
        }
        match (self, other) {
            (Self::Default, _) | (Self::Confirmed, Self::Unknown(_)) => std::cmp::Ordering::Greater,
            (Self::Unknown(unknown), Self::Unknown(other)) => {
                let now = Instant::now();
                // the newer a route is, the higher preference it has
                unknown
                    .duration_since(now)
                    .cmp(&other.duration_since(now))
                    .reverse()
            }
            _ => std::cmp::Ordering::Less,
        }
    }
}
