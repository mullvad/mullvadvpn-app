package net.mullvad.mullvadvpn.relaylist

import net.mullvad.mullvadvpn.model.GeographicLocationConstraint

data class RelayCity(
    override val name: String,
    override val code: String,
    override val location: GeographicLocationConstraint,
    override val expanded: Boolean,
    val relays: List<Relay>
) : RelayItem {
    override val type = RelayItemType.City

    override val active
        get() = relays.any { relay -> relay.active }

    override val hasChildren
        get() = relays.isNotEmpty()
}
