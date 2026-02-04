package net.mullvad.mullvadvpn.lib.common.util

import net.mullvad.mullvadvpn.lib.model.MultihopRelayListType
import net.mullvad.mullvadvpn.lib.model.RelayListType
import net.mullvad.mullvadvpn.lib.model.Settings

// If Daita is enabled without direct only we should block selection, search and hide filters for
// the multihop enry list
fun RelayListType.isEntryAndBlocked(settings: Settings?): Boolean {
    if (!isMultihopEntry()) {
        return false
    }
    return settings?.entryBlocked() == true
}

fun isEntryAndBlocked(multihopRelayListType: MultihopRelayListType, settings: Settings?): Boolean {
    if (multihopRelayListType == MultihopRelayListType.EXIT) {
        return false
    }
    return settings?.entryBlocked() == true
}

fun Settings.entryBlocked() = isDaitaEnabled() && !isDaitaDirectOnly() && isMultihopEnabled()

// If entry is blocked and we are on the exit list we should ignore any entry selection
fun ignoreEntrySelection(settings: Settings?, relayListType: RelayListType) =
    settings?.entryBlocked() == true && relayListType.isMultihopExit()

private fun RelayListType.isMultihopExit() =
    this is RelayListType.Multihop && multihopRelayListType == MultihopRelayListType.EXIT

private fun RelayListType.isMultihopEntry() =
    this is RelayListType.Multihop && multihopRelayListType == MultihopRelayListType.ENTRY
