package net.mullvad.mullvadvpn.feature.location.impl

import net.mullvad.mullvadvpn.lib.model.GeoIpLocation
import net.mullvad.mullvadvpn.lib.model.GeoLocationId
import net.mullvad.mullvadvpn.lib.model.RelayHopType
import net.mullvad.mullvadvpn.lib.model.RelayItemId
import net.mullvad.mullvadvpn.lib.model.RelayListType

internal fun GeoIpLocation.toConnectedRelay(relayListType: RelayListType): RelayItemId? =
    when (relayListType) {
        is RelayListType.Multihop if relayListType.hopType == RelayHopType.ENTRY -> {
            GeoLocationId.Hostname(
                city =
                    GeoLocationId.City(
                        country =
                            GeoLocationId.Country(code = this.entryCountryCode ?: return null),
                        code = this.entryCityCode ?: return null,
                    ),
                code = this.entryHostname ?: return null,
            )
        }
        else ->
            GeoLocationId.Hostname(
                city =
                    GeoLocationId.City(
                        country = GeoLocationId.Country(code = this.countryCode ?: return null),
                        code = this.cityCode ?: return null,
                    ),
                code = this.hostname ?: return null,
            )
    }
