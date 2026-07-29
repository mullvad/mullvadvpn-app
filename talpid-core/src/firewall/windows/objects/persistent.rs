//! Persistent and boot-time blocking filters.
//!
//! When the daemon shuts down while the firewall is in the blocked state, the ephemeral
//! filters installed by `winfw` are swapped for an equivalent set that survives both a
//! Base Filtering Engine restart and a reboot. Traffic stays blocked in the meantime,
//! including if the machine is rebooted before the daemon comes back up.
//!
//! This used to be `WINFW_CLEANUP_POLICY_CONTINUE_BLOCKING` in
//! `windows/winfw/src/winfw/winfw.cpp`, together with `rules::persistent::BlockAll`.

use std::{io, time::Duration};

use wfp::{
    ActionType, FilterBuilder, FilterEngineBuilder, FilterEnumerator, FilterLifetime, FilterWeight,
    GUID, Layer, ProviderBuilder, SubLayerBuilder, SubLayerEnumerator, Transaction, WeightRange,
    delete_filter_by_guid, delete_provider, delete_sublayer,
};
use windows_sys::Win32::Foundation::{
    FWP_E_FILTER_NOT_FOUND, FWP_E_PROVIDER_NOT_FOUND, FWP_E_SUBLAYER_NOT_FOUND,
};

use super::guids;

/// How long to wait for the WFP transaction lock before giving up.
const TRANSACTION_TIMEOUT: Duration = Duration::from_secs(5);

/// Highest weight within the sublayer. Matches `WeightClass::Max` in libwfp, which is what
/// `winfw` used for these filters.
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

/// Errors that can happen when replacing the ephemeral filters with persistent ones.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Failed to connect to the WFP filter engine
    #[error("Failed to open the WFP filter engine")]
    OpenEngine(#[source] io::Error),

    /// Failed to begin or commit the transaction
    #[error("Failed to complete the WFP transaction")]
    Transaction(#[source] io::Error),

    /// Failed to enumerate the existing filters
    #[error("Failed to enumerate WFP filters")]
    EnumerateFilters(#[source] io::Error),

    /// Failed to enumerate the existing sublayers
    #[error("Failed to enumerate WFP sublayers")]
    EnumerateSublayers(#[source] io::Error),

    /// Failed to remove an object belonging to the ephemeral provider
    #[error("Failed to remove ephemeral WFP objects")]
    RemoveEphemeral(#[source] io::Error),

    /// Failed to install one of the persistent objects
    #[error("Failed to install persistent WFP objects")]
    AddPersistent(#[source] io::Error),
}

/// Replace the ephemeral blocking filters with persistent and boot-time equivalents.
///
/// This must only be called once `winfw` has been deinitialized *without* cleaning up,
/// i.e. with [`WinFwCleanupPolicy::BlockingUntilReboot`], and only if the policy in effect
/// was the blocked one. The ephemeral filters stay in effect until this is called, and the
/// swap itself happens in a single transaction, so there is no point at which traffic is
/// permitted.
///
/// # Blocking
///
/// Waits up to [`TRANSACTION_TIMEOUT`] for the WFP transaction lock, and then as long as
/// the filter engine needs to enumerate and replace the filters.
///
/// [`WinFwCleanupPolicy::BlockingUntilReboot`]: super::super::winfw::WinFwCleanupPolicy::BlockingUntilReboot
pub fn apply_persistent_blocking() -> Result<(), Error> {
    // A standard (non-dynamic) session, so that the objects added here outlive it.
    let mut engine = FilterEngineBuilder::default()
        .transaction_timeout(TRANSACTION_TIMEOUT)
        .open()
        .map_err(Error::OpenEngine)?;

    let transaction = Transaction::new(&mut engine).map_err(Error::Transaction)?;

    remove_ephemeral_objects(&transaction)?;
    add_persistent_objects(&transaction)?;

    transaction.commit().map_err(Error::Transaction)
}

/// Remove every object belonging to [`guids::PROVIDER`], leaving the persistent provider
/// and anything not owned by us alone.
fn remove_ephemeral_objects(transaction: &Transaction<'_>) -> Result<(), Error> {
    // Collect the keys before deleting anything: objects cannot be removed while the
    // enumeration that produced them is still in progress.

    let mut filters = vec![];
    let mut enumerator = FilterEnumerator::new(transaction).map_err(Error::EnumerateFilters)?;
    while let Some(filter) = enumerator.next() {
        let filter = filter.map_err(Error::EnumerateFilters)?;
        if is_ephemeral(filter.provider()) {
            filters.push(filter.guid());
        }
    }
    drop(enumerator);

    let mut sublayers = vec![];
    let mut enumerator = SubLayerEnumerator::new(transaction).map_err(Error::EnumerateSublayers)?;
    while let Some(sublayer) = enumerator.next() {
        let sublayer = sublayer.map_err(Error::EnumerateSublayers)?;
        if is_ephemeral(sublayer.provider()) {
            sublayers.push(sublayer.guid());
        }
    }
    drop(enumerator);

    // The filters have to go first. A sublayer that still holds filters cannot be removed.
    for filter in &filters {
        ignore_not_found(
            delete_filter_by_guid(transaction, filter),
            FWP_E_FILTER_NOT_FOUND,
        )
        .map_err(Error::RemoveEphemeral)?;
    }

    for sublayer in &sublayers {
        ignore_not_found(
            delete_sublayer(transaction, sublayer),
            FWP_E_SUBLAYER_NOT_FOUND,
        )
        .map_err(Error::RemoveEphemeral)?;
    }

    ignore_not_found(
        delete_provider(transaction, &guids::PROVIDER),
        FWP_E_PROVIDER_NOT_FOUND,
    )
    .map_err(Error::RemoveEphemeral)
}

/// Whether an object with this provider is one of the ephemeral ones, i.e. one that only
/// lives for as long as the daemon is running.
fn is_ephemeral(provider: Option<GUID>) -> bool {
    provider.is_some_and(|provider| guids::eq(&provider, &guids::PROVIDER))
}

/// Install the persistent provider and sublayer, and the boot-time and persistent
/// block-all filters.
fn add_persistent_objects(transaction: &Transaction<'_>) -> Result<(), Error> {
    ProviderBuilder::default()
        .name("Mullvad VPN persistent")
        .description("Mullvad VPN firewall integration")
        .guid(guids::PROVIDER_PERSISTENT)
        .persistent()
        .add(transaction)
        .map_err(Error::AddPersistent)?;

    SubLayerBuilder::default()
        .name("Mullvad VPN persistent")
        .description("Filters that restrict traffic before WinFw is initialized")
        .guid(guids::SUBLAYER_PERSISTENT)
        .provider(guids::PROVIDER_PERSISTENT)
        .persistent()
        .weight(u16::MAX)
        .add(transaction)
        .map_err(Error::AddPersistent)?;

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
                .add(transaction)
                .map_err(Error::AddPersistent)?;
        }
    }

    Ok(())
}

/// Treat `not_found` as success. Removing an object that is already gone is not a failure.
fn ignore_not_found(result: io::Result<()>, not_found: i32) -> io::Result<()> {
    match result {
        Err(error) if error.raw_os_error() == Some(not_found) => Ok(()),
        result => result,
    }
}
