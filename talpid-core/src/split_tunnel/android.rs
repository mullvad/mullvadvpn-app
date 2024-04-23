/// Errors related to split tunneling.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Failed to update the set of excluded apps. This implies that the current
    /// tunnel was not recreated.
    #[error("Failed to update the set of excluded apps")]
    SetExcludedApps(#[source] talpid_tunnel::tun_provider::Error),
}
