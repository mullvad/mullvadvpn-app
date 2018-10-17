use std::net::IpAddr;

pub const API_HOST: &str = "api.mullvad.net";
pub const API_IP_CACHE_FILE: &str = "api-ip-address.txt";

lazy_static! {
    pub static ref API_IP: IpAddr = IpAddr::from([193, 138, 219, 46]);
}
