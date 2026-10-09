package net.mullvad.mullvadvpn.feature.location.impl

import net.mullvad.mullvadvpn.lib.model.GeoIpLocation
import net.mullvad.mullvadvpn.lib.model.RelayHopType
import net.mullvad.mullvadvpn.lib.model.RelayListType

internal fun GeoIpLocation.toConnectedHostName(relayListType: RelayListType): String? =
    when (relayListType) {
        is RelayListType.Multihop -> {
            if (relayListType.hopType == RelayHopType.ENTRY) {
                this.entryHostname
            } else {
                this.hostname
            }
        }
        RelayListType.Single -> this.hostname
    }
