//! This module defines a cache for Encrypted DNS proxy configs. The cache contains a method for
//! fetching new configs as needed.

use std::collections::HashSet;

use crate::config::ProxyConfig;
use crate::config_resolver::{self, resolve_default_config};

/// Keep track of fetched proxy configurations.
///
/// To avoid censorship and getting stuck, the proxy must have a way to efficiently try all
/// available proxies, and not get stuck on trying only a subset. [`EncryptedDnsProxyState`]
/// implements a config selection algorithm that exhaustively iterates over all available
/// proxies in an order that favours configs that are more likely to not be censored, i.e. XorV2
/// proxies, in [`Self::next_configuration`].
///
/// It is up to the consumer of [`EncryptedDnsProxyState`] to call [`Self::fetch_configs`] to fetch
/// new configs as needed, e.g. after creating the initial state.
#[derive(Debug, Default)]
pub struct EncryptedDnsProxyState {
    /// Note that we rely on the randomness of the ordering of the items in the hashset to pick a
    /// random configurations every time.
    configurations: HashSet<ProxyConfig>,
    tried_configurations: HashSet<ProxyConfig>,
}

/// Failed to fetch a proxy configuration over DNS.
#[derive(Debug)]
pub struct FetchConfigError(pub config_resolver::Error);

impl EncryptedDnsProxyState {
    /// Select a config.
    /// Always select an obfuscated configuration, if there are any left untried. If no obfuscated
    /// configurations exist, try plain configurations. The order is randomized due to the hash set
    /// storing the configurations in a random order.
    pub fn next_configuration(&mut self) -> Option<ProxyConfig> {
        if self.should_reset() {
            self.reset();
        }

        // TODO: currently, the randomized order of proxy config retrieval depends on the random
        // iteration order of a given HashSet instance. Since for now, there will be only 2
        // different configurations, it barely matters. In the future, we should use `rand`
        // instead, so that the behavior is explicit and clear.
        let selected_config = {
            // First, create an iterator for the difference between all configs and tried configs.
            let mut difference = self.configurations.difference(&self.tried_configurations);
            // Pick the first configuration if there are any. If there are none, one can only assume
            // that the configuration set is empty, so an early return is fine.
            let first_config = difference.next()?;
            // See if there are any unused obfuscated configurations in the rest of the set.
            let obfuscated_config = difference.find(|config| config.obfuscation.is_some());
            // If there is an obfuscated configuration, use that. Otherwise, use the first one.
            obfuscated_config.unwrap_or(first_config).clone()
        };

        self.tried_configurations.insert(selected_config.clone());
        Some(selected_config)
    }

    /// Fetch a config, but error out only when no existing configuration was there.
    pub async fn fetch_configs(&mut self) -> Result<(), FetchConfigError> {
        match resolve_default_config().await {
            Ok(new_configs) => {
                self.configurations = HashSet::from_iter(new_configs.into_iter());
            }
            Err(err) => {
                log::error!("Failed to fetch a new proxy configuration: {err:?}");
                if self.is_empty() {
                    return Err(FetchConfigError(err));
                }
            }
        }
        Ok(())
    }

    fn is_empty(&self) -> bool {
        self.configurations.is_empty()
    }

    /// Checks if the `tried_configurations` set should be reset.
    /// It should only be reset if the difference between `configurations` and
    /// `tried_configurations` is an empty set - in this case all available configurations have
    /// been tried.
    fn should_reset(&self) -> bool {
        self.configurations
            .difference(&self.tried_configurations)
            .count()
            == 0
    }

    /// Clears the `tried_configurations` set.
    fn reset(&mut self) {
        self.tried_configurations.clear();
    }
}
