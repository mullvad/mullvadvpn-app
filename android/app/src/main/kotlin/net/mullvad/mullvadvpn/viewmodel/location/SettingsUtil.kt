package net.mullvad.mullvadvpn.viewmodel.location

import net.mullvad.mullvadvpn.lib.model.Settings

// If Daita is enabled without direct only, it is not possible to manually select the entry
// location.
internal fun Settings.entryBlocked() =
    tunnelOptions.wireguard.daitaSettings.enabled &&
        !tunnelOptions.wireguard.daitaSettings.directOnly &&
        relaySettings.relayConstraints.wireguardConstraints.isMultihopEnabled
