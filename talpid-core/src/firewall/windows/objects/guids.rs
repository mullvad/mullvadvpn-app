//! Identifiers for the WFP objects owned by the app.
//!
//! These must match `MullvadGuids` in `windows/winfw/src/winfw/mullvadguids.cpp` exactly.
//! Changing one means objects registered by an older version are never cleaned up.

use wfp::GUID;

/// Provider owning every ephemeral object, i.e. everything that only lives for as long as
/// the daemon is running.
pub const PROVIDER: GUID = GUID::from_u128(0x21e1dab8_b9db_43c0_b343_eb9365c7bdd2);

/// Provider owning the persistent and boot-time objects, which outlive the daemon.
pub const PROVIDER_PERSISTENT: GUID = GUID::from_u128(0x2bc5bc63_80b0_4119_86d3_6afe0dff2a26);

/// Sublayer holding the persistent and boot-time rules.
pub const SUBLAYER_PERSISTENT: GUID = GUID::from_u128(0x3c28881e_8891_4d61_b87f_f272502d1005);

/// Boot-time block-all filters, ordered as they are installed.
pub const FILTER_BOOTTIME_BLOCK_ALL: [GUID; 4] = [
    // Outbound, IPv4
    GUID::from_u128(0x5996aa42_102b_419f_ad3d_835db50b8b01),
    // Inbound, IPv4
    GUID::from_u128(0x6150b73d_4dfa_4c30_80eb_e0ee535193da),
    // Outbound, IPv6
    GUID::from_u128(0x139b8b26_5037_4929_9237_e873bddd651d),
    // Inbound, IPv6
    GUID::from_u128(0x129927e2_7a3a_49bb_b987_3692563a83f4),
];

/// Persistent block-all filters, ordered as they are installed.
pub const FILTER_PERSISTENT_BLOCK_ALL: [GUID; 4] = [
    // Outbound, IPv4
    GUID::from_u128(0x79860c64_9a5e_48a3_b5f3_d64b41659aa5),
    // Inbound, IPv4
    GUID::from_u128(0x9f177f14_f090_4fde_98f9_84153125a7c5),
    // Outbound, IPv6
    GUID::from_u128(0xa9b72749_b1c1_4483_a371_90e18668532e),
    // Inbound, IPv6
    GUID::from_u128(0x333e7e5c_9293_4bda_8b19_b670191cc47c),
];

/// Compare two GUIDs. [GUID] does not implement [PartialEq].
pub fn eq(left: &GUID, right: &GUID) -> bool {
    left.data1 == right.data1
        && left.data2 == right.data2
        && left.data3 == right.data3
        && left.data4 == right.data4
}
