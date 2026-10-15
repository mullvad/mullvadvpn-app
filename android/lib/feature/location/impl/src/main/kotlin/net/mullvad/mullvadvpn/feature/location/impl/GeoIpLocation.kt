package net.mullvad.mullvadvpn.feature.location.impl

import net.mullvad.mullvadvpn.lib.model.GeoIpLocation
import net.mullvad.mullvadvpn.lib.model.RelayHopType
import net.mullvad.mullvadvpn.lib.model.RelayItemId
import net.mullvad.mullvadvpn.lib.model.RelayListType

internal fun GeoIpLocation.connectedLocationIn(relayListType: RelayListType): RelayItemId? =
    when (relayListType) {
        is RelayListType.Multihop if relayListType.hopType == RelayHopType.ENTRY -> {
            this.entryLocation
        }
        else -> this.location
    }
