use std::net::IpAddr;

pub const AM_I_MULLVAD_HOST: &str = "am.i.mullvad.net";
pub const AM_I_MULLVAD_IP_CACHE_FILE: &str = "am-i-mullvad-ip-address.txt";

pub const API_HOST: &str = "api.mullvad.net";
pub const API_IP_CACHE_FILE: &str = "api-ip-address.txt";

lazy_static! {
    pub static ref AM_I_MULLVAD_IP: IpAddr = IpAddr::from([46, 166, 184, 225]);
    pub static ref API_IP: IpAddr = IpAddr::from([193, 138, 219, 46]);
}
