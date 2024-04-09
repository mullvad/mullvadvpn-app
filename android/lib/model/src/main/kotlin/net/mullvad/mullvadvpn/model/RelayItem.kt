package net.mullvad.mullvadvpn.model

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
        override val expanded: Boolean,
        val id: CustomListId,
        val locations: List<Location>,
    ) : RelayItem {
        override val active
            get() = locations.any { location -> location.active }

        override val hasChildren
            get() = locations.isNotEmpty()

        override val code = id.value
    }

    sealed interface Location : RelayItem {
        val location: GeographicLocationConstraint

        data class Country(
            override val name: String,
            override val code: String,
            override val expanded: Boolean,
            val cities: List<City>
        ) : Location {
            override val location = GeographicLocationConstraint.Country(code)
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
            override val location: GeographicLocationConstraint.City,
            val relays: List<Relay>
        ) : Location {

            override val active
                get() = relays.any { relay -> relay.active }

            override val hasChildren
                get() = relays.isNotEmpty()
        }

        data class Relay(
            override val name: String,
            override val locationName: String,
            override val active: Boolean,
            override val location: GeographicLocationConstraint.Hostname,
            val provider: String,
            val ownership: Ownership
        ) : Location {
            override val code = name
            override val hasChildren = false
            override val expanded = false
        }
    }
}
