package net.mullvad.mullvadvpn.lib.model

sealed interface RecentItem {
    data class Relay(val item: RelayItem) : RecentItem

    data object Automatic : RecentItem
}

sealed interface Recents {
    data object Disabled : Recents

    data class Enabled(val entry: List<EntryRecent>, val exit: List<ExitRecent>) : Recents
}

sealed interface EntryRecent {
    data object Automatic : EntryRecent

    data class Location(val location: RelayItemId) : EntryRecent
}

data class ExitRecent(val location: RelayItemId)
