package net.mullvad.mullvadvpn.lib.usecase

import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.map
import net.mullvad.mullvadvpn.lib.common.util.relaylist.withDescendants
import net.mullvad.mullvadvpn.lib.model.ConnectionPath
import net.mullvad.mullvadvpn.lib.model.LatLong
import net.mullvad.mullvadvpn.lib.model.Latitude
import net.mullvad.mullvadvpn.lib.model.Longitude
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.lib.model.TunnelState
import net.mullvad.mullvadvpn.lib.repository.ConnectionProxy
import net.mullvad.mullvadvpn.lib.repository.RelayListRepository

class ConnectionPathUseCase(
    val lastKnownDisconnectedLocation: LastKnownLocationUseCase,
    val connectionProxy: ConnectionProxy,
    val relayListRepository: RelayListRepository,
) {
    operator fun invoke(): Flow<ConnectionPath> =
        combine(
            lastKnownDisconnectedLocation.lastKnownDisconnectedLocation.map {
                it?.let {
                    LatLong(
                        Latitude.fromFloat(it.latitude.toFloat()),
                        Longitude.fromFloat(it.longitude.toFloat()),
                    )
                }
            },
            connectionProxy.tunnelState,
            relayListRepository.relayList,
        ) { offlineLocation, tunnelState, relayList ->
            val (entryHostname, exitHostname) =
                when (tunnelState) {
                    is TunnelState.Connected ->
                        tunnelState.location?.entryHostname to tunnelState.location?.hostname
                    is TunnelState.Connecting -> null to null
                    is TunnelState.Disconnected -> null to null
                    is TunnelState.Disconnecting -> null to null
                    is TunnelState.Error -> null to null
                }

            val entryRelay = entryHostname?.let { relayList.findRelay(it) }
            val exitRelay = exitHostname?.let { relayList.findRelay(it) }

            ConnectionPath(offlineLocation, entryRelay?.latLong, exitRelay?.latLong)
        }

    private fun List<RelayItem.Location.Country>.findRelay(
        hostname: String
    ): RelayItem.Location.Relay? =
        withDescendants().filterIsInstance<RelayItem.Location.Relay>().firstOrNull {
            it.id.code == hostname
        }
}
