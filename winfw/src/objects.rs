//! The structural WFP objects the firewall registers, and their lifecycle.

use std::io;

use wfp::{
    FilterEnumerator, SubLayerEnumerator, Transaction, delete_filter_by_guid, delete_provider,
    delete_sublayer,
};
use windows_sys::Win32::Foundation::{
    FWP_E_FILTER_NOT_FOUND, FWP_E_PROVIDER_NOT_FOUND, FWP_E_SUBLAYER_NOT_FOUND,
};

use crate::{
    guids::{PROVIDER, PROVIDER_PERSISTENT, SUBLAYER_BASELINE, SUBLAYER_DNS},
    session::{Record, Recorder},
};

/// Install the Mullvad WFP provider.
pub fn add_provider(transaction: &Transaction<'_>, recorder: &mut Recorder) -> io::Result<()> {
    wfp::ProviderBuilder::default()
        .name("Mullvad VPN")
        .description("Mullvad VPN firewall integration")
        .guid(PROVIDER)
        .add(transaction)?;
    recorder.record(Record::Provider(PROVIDER));
    Ok(())
}

/// Install the baseline sublayer, which holds every ordinary firewall rule.
pub fn add_sublayer_baseline(
    transaction: &Transaction<'_>,
    recorder: &mut Recorder,
) -> io::Result<()> {
    wfp::SubLayerBuilder::default()
        .name("Mullvad VPN baseline")
        .description("Filters that enforce a good baseline")
        .weight(u16::MAX)
        .guid(SUBLAYER_BASELINE)
        .provider(PROVIDER)
        .add(transaction)?;
    recorder.record(Record::Sublayer(SUBLAYER_BASELINE));
    Ok(())
}

/// Install the DNS sublayer, which holds the rules that restrict DNS.
pub fn add_sublayer_dns(transaction: &Transaction<'_>, recorder: &mut Recorder) -> io::Result<()> {
    wfp::SubLayerBuilder::default()
        .name("Mullvad VPN DNS")
        .description("Filters that restrict DNS traffic")
        .weight(u16::MAX - 1)
        .guid(SUBLAYER_DNS)
        .provider(PROVIDER)
        .add(transaction)?;
    recorder.record(Record::Sublayer(SUBLAYER_DNS));
    Ok(())
}

/// Uninstall every WFP object owned by the firewall, both the ephemeral and the
/// persistent ones. Used at initialization, to clean up anything left behind
/// by a previous run, and by the reset API when the module is not initialized.
///
/// Since the session cannot assume anything about which objects are already
/// installed, everything is discovered by enumeration. Deleting sublayers and
/// providers can fail transiently while filters in them are being deleted, so
/// filters are deleted first, then sublayers, then providers.
pub fn purge_all(transaction: &Transaction<'_>) -> io::Result<()> {
    let mut filters = Vec::new();
    let mut sublayers = Vec::new();

    let mut filter_enum = FilterEnumerator::new(transaction)?;
    while let Some(filter) = filter_enum.next() {
        let filter = filter?;
        if is_mullvad_provider(filter.provider().as_ref()) {
            filters.push(filter.guid());
        }
    }

    let mut sublayer_enum = SubLayerEnumerator::new(transaction)?;
    while let Some(sublayer) = sublayer_enum.next() {
        let sublayer = sublayer?;
        if is_mullvad_provider(sublayer.provider().as_ref()) {
            sublayers.push(sublayer.guid());
        }
    }

    for guid in filters {
        ignore_not_found(delete_filter_by_guid(transaction, guid))?;
    }

    for guid in sublayers {
        ignore_not_found(delete_sublayer(transaction, guid))?;
    }

    ignore_not_found(delete_provider(transaction, PROVIDER))?;
    ignore_not_found(delete_provider(transaction, PROVIDER_PERSISTENT))?;

    Ok(())
}

/// Treat the WFP "object not found" errors as success: deleting
/// fire-and-forget is best effort, since another process may have removed the
/// objects already.
pub(crate) fn ignore_not_found(result: io::Result<()>) -> io::Result<()> {
    match result {
        Ok(()) => Ok(()),
        Err(error) => {
            if error.raw_os_error() == Some(FWP_E_FILTER_NOT_FOUND)
                || error.raw_os_error() == Some(FWP_E_SUBLAYER_NOT_FOUND)
                || error.raw_os_error() == Some(FWP_E_PROVIDER_NOT_FOUND)
            {
                Ok(())
            } else {
                Err(error)
            }
        }
    }
}

fn is_mullvad_provider(provider: Option<&wfp::Guid>) -> bool {
    provider.is_some_and(|guid| *guid == PROVIDER || *guid == PROVIDER_PERSISTENT)
}
