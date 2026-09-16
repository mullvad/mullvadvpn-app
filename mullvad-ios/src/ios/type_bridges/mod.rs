use std::net::IpAddr;

pub struct UniIpAddr(pub IpAddr);
uniffi::custom_type!(UniIpAddr, String, {
    lower: |time_interval| time_interval.0.to_string(),
    try_lift: |val| Ok(UniIpAddr(val.parse().unwrap()))
});

pub type AmIMullvad = mullvad_types::location::AmIMullvad;

#[derive(uniffi::Record)]
#[uniffi(name = "AmIMullvadResponse")]
pub struct AmIMullvadResponse {
    pub ip: UniIpAddr,
    pub country: String,
    pub city: Option<String>,
    pub latitude: f64,
    pub longitude: f64,
    pub mullvad_exit_ip: bool,
}

impl From<AmIMullvad> for AmIMullvadResponse {
    fn from(value: AmIMullvad) -> Self {
        AmIMullvadResponse {
            latitude: value.latitude,
            longitude: value.longitude,
            mullvad_exit_ip: value.mullvad_exit_ip,
            country: value.country,
            city: value.city,
            ip: UniIpAddr(value.ip),
        }
    }
}
