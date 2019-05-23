package net.mullvad.mullvadvpn.relaylist

import net.mullvad.mullvadvpn.model.LocationConstraint

data class Relay(
    val countryCode: String,
    val cityCode: String,
    override val name: String
) : RelayItem {
    override val code = name
    override val type = RelayItemType.Relay
    override val location = LocationConstraint.Hostname(countryCode, cityCode, name)
    override val hasChildren = false

    override val visibleChildCount = 0

    override var expanded
        get() = false
        set(value) {}
}
