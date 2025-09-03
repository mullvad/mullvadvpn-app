package net.mullvad.mullvadvpn.viewmodel.location

import net.mullvad.mullvadvpn.compose.state.MultihopRelayListType
import net.mullvad.mullvadvpn.compose.state.RelayListType
import net.mullvad.mullvadvpn.lib.model.Settings

// If Daita is enabled without direct only we should block selection, search and hide filters for
// the multihop enry list
internal fun RelayListType.isEntryAndBlocked(settings: Settings?): Boolean {
    val isMultihopEntry = isMultihopEntry()

    if (!isMultihopEntry) {
        return false
    }

    return settings?.entryBlocked() == true
}

private fun Settings.entryBlocked() =
    tunnelOptions.wireguard.daitaSettings.enabled &&
        !tunnelOptions.wireguard.daitaSettings.directOnly &&
        relaySettings.relayConstraints.wireguardConstraints.isMultihopEnabled

private fun RelayListType.isMultihopEntry() =
    this is RelayListType.Multihop && multihopRelayListType == MultihopRelayListType.ENTRY
