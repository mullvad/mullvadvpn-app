package net.mullvad.mullvadvpn.relaylist

import net.mullvad.mullvadvpn.model.GeoIpLocation
import net.mullvad.mullvadvpn.model.GeographicLocationConstraint

sealed interface RelayItem {
    val name: String
    val code: String
    val active: Boolean
    val hasChildren: Boolean

    val locationName: String
        get() = name

    val expanded: Boolean

    data class CustomList(
        override val name: String,
        override val code: String,
        override val expanded: Boolean,
        val id: String,
        val locations: List<RelayItem>,
    ) : RelayItem {
        override val active
            get() = locations.any { location -> location.active }

        override val hasChildren
            get() = locations.isNotEmpty()
    }

    data class Country(
        override val name: String,
        override val code: String,
        override val expanded: Boolean,
        val cities: List<City>
    ) : RelayItem {
        val location = GeographicLocationConstraint.Country(code)
        override val active
            get() = cities.any { city -> city.active }

        override val hasChildren
            get() = cities.isNotEmpty()
    }

    data class City(
        override val name: String,
        override val code: String,
        override val expanded: Boolean,
        val location: GeographicLocationConstraint.City,
        val relays: List<Relay>
    ) : RelayItem {

        override val active
            get() = relays.any { relay -> relay.active }

        override val hasChildren
            get() = relays.isNotEmpty()
    }

    data class Relay(
        override val name: String,
        override val locationName: String,
        override val active: Boolean,
        val location: GeographicLocationConstraint.Hostname,
    ) : RelayItem {
        override val code = name
        override val hasChildren = false
        override val expanded = false
    }

    fun location(): GeoIpLocation? {
        return when (this) {
            is CustomList -> null
            is Country -> location.location
            is City -> location.location
            is Relay -> location.location
        }
    }
}
