package net.mullvad.mullvadvpn.lib.model

sealed interface RelayItemSelection {
    val exitLocation: Constraint<RelayItemId>

    data class Single(override val exitLocation: Constraint<RelayItemId>) : RelayItemSelection

    data class Multiple(
        val entryLocation: Constraint<RelayItemId>,
        override val exitLocation: Constraint<RelayItemId>,
    ) : RelayItemSelection
}
