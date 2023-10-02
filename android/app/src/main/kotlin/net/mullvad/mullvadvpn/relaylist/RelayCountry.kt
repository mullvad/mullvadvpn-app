package net.mullvad.mullvadvpn.relaylist

import net.mullvad.mullvadvpn.model.GeographicLocationConstraint

data class RelayCountry(
    override val name: String,
    override val code: String,
    override val expanded: Boolean,
    val cities: List<RelayCity>
) : RelayItem {
    override val type = RelayItemType.Country
    override val location = GeographicLocationConstraint.Country(code)

    override val active
        get() = cities.any { city -> city.active }

    override val hasChildren
        get() = cities.isNotEmpty()
}
