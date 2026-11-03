//! Persistent and boot-time blocking filters.
//!
//! When the daemon shuts down while the firewall is in the blocked state, the ephemeral
//! filters are swapped for an equivalent set that survives both a Base Filtering Engine
//! restart and a reboot. Traffic stays blocked in the meantime, including if the machine
//! is rebooted before the daemon comes back up.

use std::io;

use wfp::{
    ActionType, FilterBuilder, FilterLifetime, FilterWeight, Layer, ProviderBuilder,
    SubLayerBuilder, Transaction, WeightRange,
};

use crate::{guids, objects};

/// Highest weight within the sublayer.
const MAX_WEIGHT: u8 = 15;

const FILTER_DESCRIPTION: &str =
    "This filter is part of a rule that restricts inbound and outbound traffic";

/// The block-all filters, in the order they are installed. Both the boot-time and the
/// persistent set consist of these same four filters.
const BLOCK_ALL_FILTERS: [(&str, Layer); 4] = [
    ("Block all outbound connections (IPv4)", Layer::ConnectV4),
    ("Block all inbound connections (IPv4)", Layer::AcceptV4),
    ("Block all outbound connections (IPv6)", Layer::ConnectV6),
    ("Block all inbound connections (IPv6)", Layer::AcceptV6),
];

/// Replace the ephemeral objects with persistent and boot-time blocking filters.
///
/// The swap happens within `transaction`, so there is no point at which traffic is
/// permitted. The session must be a standard (non-dynamic) one, so that the objects
/// outlive it.
pub fn replace_ephemeral_objects(transaction: &Transaction<'_>) -> io::Result<()> {
    objects::purge_providers(transaction, &[guids::PROVIDER])?;
    add_persistent_objects(transaction)
}

/// Install the persistent provider and sublayer, and the boot-time and persistent
/// block-all filters.
fn add_persistent_objects(transaction: &Transaction<'_>) -> io::Result<()> {
    ProviderBuilder::default()
        .name("Mullvad VPN persistent")
        .description("Mullvad VPN firewall integration")
        .guid(guids::PROVIDER_PERSISTENT)
        .persistent()
        .add(transaction)?;

    SubLayerBuilder::default()
        .name("Mullvad VPN persistent")
        .description("Filters that restrict traffic before WinFw is initialized")
        .guid(guids::SUBLAYER_PERSISTENT)
        .provider(guids::PROVIDER_PERSISTENT)
        .persistent()
        .weight(u16::MAX)
        .add(transaction)?;

    let weight = FilterWeight::Range(
        WeightRange::try_from(MAX_WEIGHT).expect("MAX_WEIGHT is a valid filter weight"),
    );

    // Boot-time filters are applied before the Base Filtering Engine starts, persistent
    // ones once it has started. Both are needed to keep traffic blocked across a reboot.
    let filters = [
        (&guids::FILTER_BOOTTIME_BLOCK_ALL, FilterLifetime::Boottime),
        (
            &guids::FILTER_PERSISTENT_BLOCK_ALL,
            FilterLifetime::Persistent,
        ),
    ];

    for (keys, lifetime) in filters {
        for (key, (name, layer)) in keys.iter().zip(BLOCK_ALL_FILTERS) {
            FilterBuilder::default()
                .name(name)
                .description(FILTER_DESCRIPTION)
                .guid(*key)
                .provider(guids::PROVIDER_PERSISTENT)
                .sublayer(guids::SUBLAYER_PERSISTENT)
                .layer(layer)
                .weight(weight)
                .lifetime(lifetime)
                .action(ActionType::Block)
                .add(transaction)?;
        }
    }

    Ok(())
}
