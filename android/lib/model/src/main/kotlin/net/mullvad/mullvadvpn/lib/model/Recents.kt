package net.mullvad.mullvadvpn.lib.model

sealed interface Recents {
    data object Disabled : Recents

    data class Enabled(val recents: List<Recent>) : Recents
}

sealed interface Recent {
    data class Singlehop(val location: RelayItemId) : Recent

    data class Multihop(val entry: RelayItemId, val exit: RelayItemId) : Recent
}
