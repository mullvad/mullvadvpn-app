package net.mullvad.mullvadvpn.usecase

import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.filterIsInstance
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import net.mullvad.mullvadvpn.model.GeoIpLocation
import net.mullvad.mullvadvpn.model.TunnelState
import net.mullvad.mullvadvpn.ui.serviceconnection.ConnectionProxy

class LastKnownLocationUseCase(
    connectionProxy: ConnectionProxy,
    dispatcher: CoroutineDispatcher = Dispatchers.IO,
) {
    val lastKnownDisconnectedLocation: Flow<GeoIpLocation?> =
        connectionProxy.tunnelState
            .filterIsInstance<TunnelState.Disconnected>()
            .map { it.location }
            .filterNotNull()
            .stateIn(CoroutineScope(dispatcher), SharingStarted.Lazily, null)
}
