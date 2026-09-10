// Generated from extra-lan-networks.txt by `cargo run -p talpid-types --bin generate-extra-lan-nets`. Do not edit.
/// Networks listed in `extra-lan-networks.txt`, appended to [`BASE_LAN_NETS`] to form [`ALLOWED_LAN_NETS`].
pub const EXTRA_LAN_NETS: [IpNetwork; 1] = [
    v4(Ipv4Addr::new(100, 64, 0, 0), 10),
];
