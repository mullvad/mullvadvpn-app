package net.mullvad.mullvadvpn.lib.model

import arrow.optics.optics

typealias DomainCustomList = CustomList

sealed interface HopSelection {
    data class Single(val relay: Constraint<RelayItem>?) : HopSelection

    data class Multi(val entry: Constraint<RelayItem>?, val exit: Constraint<RelayItem>?) :
        HopSelection

    fun entry() =
        when (this) {
            is Multi -> entry
            is Single -> relay
        }

    fun exit() =
        when (this) {
            is Multi -> exit
            is Single -> relay
        }
}

@optics
sealed interface RelayItem {
    val id: RelayItemId
    val name: String

    val active: Boolean
    val hasChildren: Boolean

    @optics
    data class CustomList(val customList: DomainCustomList, val locations: List<Location>) :
        RelayItem {
        override val name: String = customList.name.value
        override val id = customList.id

        override val active = locations.any { it.active }
        override val hasChildren: Boolean = locations.isNotEmpty()

        companion object
    }

    @optics
    sealed interface Location : RelayItem {
        override val id: GeoLocationId

        @optics
        data class Country(
            override val id: GeoLocationId.Country,
            override val name: String,
            val cities: List<City>,
        ) : Location {
            val relays = cities.flatMap { city -> city.relays }
            override val active = cities.any { it.active }
            override val hasChildren: Boolean = cities.isNotEmpty()

            companion object
        }

        @optics
        data class City(
            override val id: GeoLocationId.City,
            override val name: String,
            val relays: List<Relay>,
        ) : Location {
            override val active = relays.any { it.active }
            override val hasChildren: Boolean = relays.isNotEmpty()

            companion object
        }

        @optics
        data class Relay(
            override val id: GeoLocationId.Hostname,
            val provider: ProviderId,
            val ownership: Ownership,
            override val active: Boolean,
            val daita: Boolean,
            val quic: Quic?,
            val lwo: Boolean,
        ) : Location {
            override val name: String = id.code
            override val hasChildren: Boolean = false

            companion object
        }

        companion object
    }

    companion object
}
