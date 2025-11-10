package net.mullvad.mullvadvpn.viewmodel.location

import net.mullvad.mullvadvpn.compose.state.MultihopRelayListType
import net.mullvad.mullvadvpn.compose.state.RelayListType
import net.mullvad.mullvadvpn.lib.model.Settings
import net.mullvad.mullvadvpn.util.isDaitaDirectOnly
import net.mullvad.mullvadvpn.util.isDaitaEnabled
import net.mullvad.mullvadvpn.util.isMultihopEnabled

// If Daita is enabled without direct only we should block selection, search and hide filters for
// the multihop enry list
internal fun RelayListType.isEntryAndBlocked(settings: Settings?): Boolean {
    val isMultihopEntry = isMultihopEntry()

    if (!isMultihopEntry) {
        return false
    }

    return settings?.entryBlocked() == true
}

internal fun Settings.entryBlocked() =
    isDaitaEnabled() && !isDaitaDirectOnly() && isMultihopEnabled()

// If entry is blocked and we are on the exit list we should ignore any entry selection
internal fun ignoreEntrySelection(settings: Settings?, relayListType: RelayListType) =
    settings?.entryBlocked() == true && relayListType.isMultihopExit()

private fun RelayListType.isMultihopExit() =
    this is RelayListType.Multihop && multihopRelayListType == MultihopRelayListType.EXIT

private fun RelayListType.isMultihopEntry() =
    this is RelayListType.Multihop && multihopRelayListType == MultihopRelayListType.ENTRY
