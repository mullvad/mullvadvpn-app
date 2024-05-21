package net.mullvad.mullvadvpn.lib.model

import android.os.Parcelable
import arrow.optics.optics
import kotlinx.parcelize.Parcelize

@optics
sealed interface RelayItemId : Parcelable {
    companion object
}

@optics
@Parcelize
@JvmInline
value class CustomListId(val value: String) :
    net.mullvad.mullvadvpn.lib.model.RelayItemId, Parcelable {
    companion object
}

@optics
sealed interface GeoLocationId : net.mullvad.mullvadvpn.lib.model.RelayItemId {
    @optics
    @Parcelize
    data class Country(val countryCode: String) : net.mullvad.mullvadvpn.lib.model.GeoLocationId {
        companion object
    }

    @optics
    @Parcelize
    data class City(
        val countryCode: net.mullvad.mullvadvpn.lib.model.GeoLocationId.Country,
        val cityCode: String
    ) : net.mullvad.mullvadvpn.lib.model.GeoLocationId {
        companion object
    }

    @optics
    @Parcelize
    data class Hostname(
        val city: net.mullvad.mullvadvpn.lib.model.GeoLocationId.City,
        val hostname: String
    ) : net.mullvad.mullvadvpn.lib.model.GeoLocationId {
        companion object
    }

    val country: net.mullvad.mullvadvpn.lib.model.GeoLocationId.Country
        get() =
            when (this) {
                is net.mullvad.mullvadvpn.lib.model.GeoLocationId.Country -> this
                is net.mullvad.mullvadvpn.lib.model.GeoLocationId.City -> this.countryCode
                is net.mullvad.mullvadvpn.lib.model.GeoLocationId.Hostname -> this.city.countryCode
            }

    companion object
}
