package net.mullvad.mullvadvpn.relaylist

import net.mullvad.mullvadvpn.model.LocationConstraint

data class Relay(
    val city: RelayCity,
    override val name: String,
    override val active: Boolean
) : RelayItem {
    override val code = name
    override val type = RelayItemType.Relay
    override val location = LocationConstraint.Hostname(city.country.code, city.code, name)
    override val hasChildren = false

    override val visibleChildCount = 0

    override val locationName = "${city.name} ($name)"

    override var expanded
        get() = false
        set(_) {}
}
