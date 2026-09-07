use std::net::IpAddr;

pub struct UniIpAddr(pub IpAddr);
uniffi::custom_type!(UniIpAddr, String, {
    lower: |time_interval| time_interval.0.to_string(),
    try_lift: |val| Ok(UniIpAddr(val.parse().unwrap()))
});
