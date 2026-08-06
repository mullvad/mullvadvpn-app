package net.mullvad.mullvadvpn.lib.common.util

import net.mullvad.mullvadvpn.lib.model.MultihopMode
import net.mullvad.mullvadvpn.lib.model.RelayHopType
import net.mullvad.mullvadvpn.lib.model.RelayListType
import net.mullvad.mullvadvpn.lib.model.Settings

// If we are doing an automatic multihop we should block selection, search and hide filters for
// the multihop entry list for the multihop entry.
fun RelayListType.isEntryAndBlocked(settings: Settings?): Boolean {
    if (!isMultihopEntry()) {
        return false
    }
    return settings?.isEntryBlocked() == true
}

fun isEntryAndBlocked(hopType: RelayHopType, settings: Settings?): Boolean {
    if (hopType == RelayHopType.EXIT) {
        return false
    }
    return settings?.isEntryBlocked() == true
}

fun Settings.isEntryBlocked() = multihopMode() == MultihopMode.WHEN_NEEDED

// If entry is blocked and we are on the exit list we should ignore any entry selection
fun ignoreEntrySelection(settings: Settings?, relayListType: RelayListType) =
    settings?.isEntryBlocked() == true && relayListType.isMultihopExit()

private fun RelayListType.isMultihopExit() =
    this is RelayListType.Multihop && hopType == RelayHopType.EXIT

private fun RelayListType.isMultihopEntry() =
    this is RelayListType.Multihop && hopType == RelayHopType.ENTRY
