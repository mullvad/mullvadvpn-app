package net.mullvad.mullvadvpn.relaylist

import net.mullvad.mullvadvpn.model.CustomListName
import net.mullvad.mullvadvpn.model.CustomListId
import net.mullvad.mullvadvpn.model.GeographicLocationConstraint
import net.mullvad.mullvadvpn.model.Ownership

sealed interface RelayItem {
    val name: String
    val code: String
    val active: Boolean
    val hasChildren: Boolean

    val locationName: String
        get() = name

    val expanded: Boolean

    data class CustomList(
        val customListName: CustomListName,
        override val expanded: Boolean,
        val id: CustomListId,
        val locations: List<RelayItem>,
    ) : RelayItem {
        override val name: String = customListName.value
        override val active
            get() = locations.any { location -> location.active }

        override val hasChildren
            get() = locations.isNotEmpty()

        override val code = id.value
    }

    data class Country(
        override val name: String,
        override val code: String,
        override val expanded: Boolean,
        val cities: List<City>
    ) : RelayItem {
        val location = GeographicLocationConstraint.Country(code)
        val relays = cities.flatMap { city -> city.relays }
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
        val providerName: String,
        val ownership: Ownership,
    ) : RelayItem {
        override val code = name
        override val hasChildren = false
        override val expanded = false
    }

    fun location(): GeographicLocationConstraint? {
        return when (this) {
            is CustomList -> null
            is Country -> location
            is City -> location
            is Relay -> location
        }
    }
}
