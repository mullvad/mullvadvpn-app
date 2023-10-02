package net.mullvad.mullvadvpn.relaylist

import net.mullvad.mullvadvpn.model.GeographicLocationConstraint

data class Relay(
    override val name: String,
    override val location: GeographicLocationConstraint,
    override val locationName: String,
    override val active: Boolean
) : RelayItem {
    override val code = name
    override val type = RelayItemType.Relay
    override val hasChildren = false

    override val expanded = false
}
