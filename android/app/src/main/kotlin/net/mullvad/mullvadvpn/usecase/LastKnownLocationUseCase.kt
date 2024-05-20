package net.mullvad.mullvadvpn.usecase

import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.filterIsInstance
import kotlinx.coroutines.flow.mapNotNull
import kotlinx.coroutines.flow.stateIn
import net.mullvad.mullvadvpn.lib.shared.ConnectionProxy
import net.mullvad.mullvadvpn.model.GeoIpLocation
import net.mullvad.mullvadvpn.model.TunnelState

class LastKnownLocationUseCase(
    connectionProxy: ConnectionProxy,
    dispatcher: CoroutineDispatcher = Dispatchers.IO,
) {
    val lastKnownDisconnectedLocation: Flow<GeoIpLocation?> =
        connectionProxy.tunnelState
            .filterIsInstance<TunnelState.Disconnected>()
            .mapNotNull { it.location }
            .stateIn(CoroutineScope(dispatcher), SharingStarted.Lazily, null)
}
