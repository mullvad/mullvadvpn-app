package net.mullvad.mullvadvpn.lib.model

sealed interface RecentItem {
    data class Relay(val item: RelayItem) : RecentItem

    data object Automatic : RecentItem
}

sealed interface Recents {
    data object Disabled : Recents

    data class Enabled(val recents: List<Recent>) : Recents
}

sealed interface Recent {
    data class Singlehop(val location: RelayItemId) : Recent

    data class Multihop(val entry: RelayItemId, val exit: RelayItemId) : Recent

    data class AutomaticEntryMultihop(val exit: RelayItemId) : Recent
}
