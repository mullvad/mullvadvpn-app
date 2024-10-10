package net.mullvad.mullvadvpn.lib.model

sealed interface SelectedLocation {
    val exitLocation: Constraint<RelayItemId>

    data class Single(override val exitLocation: Constraint<RelayItemId>) : SelectedLocation

    data class Multiple(
        val entryLocation: Constraint<RelayItemId>,
        override val exitLocation: Constraint<RelayItemId>,
    ) : SelectedLocation
}
