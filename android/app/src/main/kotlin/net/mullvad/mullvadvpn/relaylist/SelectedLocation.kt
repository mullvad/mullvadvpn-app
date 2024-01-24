package net.mullvad.mullvadvpn.relaylist

import net.mullvad.mullvadvpn.model.GeographicLocationConstraint

data class SelectedLocation(
    val id: String,
    val name: String,
    val geographicLocationConstraint: GeographicLocationConstraint?
)
