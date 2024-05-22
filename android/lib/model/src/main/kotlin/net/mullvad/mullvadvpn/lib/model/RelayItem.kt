package net.mullvad.mullvadvpn.lib.model

import arrow.optics.optics

@optics
sealed interface RelayItem {
    val id: RelayItemId
    val name: String
    val active: Boolean
    val hasChildren: Boolean
    val expanded: Boolean

    @optics
    data class CustomList(
        override val id: CustomListId,
        val customListName: CustomListName,
        val locations: List<Location>,
        override val expanded: Boolean,
    ) : RelayItem {
        override val name: String = customListName.value

        override val active
            get() = locations.any { location -> location.active }

        override val hasChildren
            get() = locations.isNotEmpty()

        companion object
    }

    @optics
    sealed interface Location : RelayItem {
        override val id: GeoLocationId

        @optics
        data class Country(
            override val id: GeoLocationId.Country,
            override val name: String,
            override val expanded: Boolean,
            val cities: List<City>
        ) : Location {
            val relays = cities.flatMap { city -> city.relays }

            override val active
                get() = cities.any { city -> city.active }

            override val hasChildren
                get() = cities.isNotEmpty()

            companion object
        }

        @optics
        data class City(
            override val id: GeoLocationId.City,
            override val name: String,
            override val expanded: Boolean,
            val relays: List<Relay>
        ) : Location {

            override val active
                get() = relays.any { relay -> relay.active }

            override val hasChildren
                get() = relays.isNotEmpty()

            companion object
        }

        @optics
        data class Relay(
            override val id: GeoLocationId.Hostname,
            val provider: Provider,
            override val active: Boolean,
        ) : Location {
            override val name: String = id.hostname

            override val hasChildren = false
            override val expanded = false

            companion object
        }

        companion object
    }

    companion object
}
