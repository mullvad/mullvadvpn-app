package net.mullvad.mullvadvpn.lib.repository

import android.content.Context
import arrow.core.Either
import arrow.core.raise.either
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.update
import net.mullvad.mullvadvpn.lib.common.util.prepareVpnSafe
import net.mullvad.mullvadvpn.lib.grpc.ManagementService
import net.mullvad.mullvadvpn.lib.model.ConnectError
import net.mullvad.mullvadvpn.lib.model.DisconnectReason
import net.mullvad.mullvadvpn.lib.model.GeoIpLocation
import net.mullvad.mullvadvpn.lib.model.TunnelState

class ConnectionProxy(
    private val context: Context,
    private val managementService: ManagementService,
    translationRepository: RelayLocationTranslationRepository,
) {
    private val _connectionRequests = MutableStateFlow(0L)

    // Tracks attempts even when the daemon skips or conflates intermediate tunnel states.
    val connectionRequests: StateFlow<Long> = _connectionRequests.asStateFlow()

    val tunnelState =
        combine(managementService.tunnelState, translationRepository.translations) {
            tunnelState,
            translations ->
            tunnelState.translateLocations(translations)
        }

    private fun TunnelState.translateLocations(translations: Map<String, String>): TunnelState {
        return when (this) {
            is TunnelState.Connecting -> copy(location = location?.translate(translations))
            is TunnelState.Disconnected -> copy(location = location?.translate(translations))
            is TunnelState.Disconnecting -> this
            is TunnelState.Error -> this
            is TunnelState.Connected -> copy(location = location?.translate(translations))
        }
    }

    private fun GeoIpLocation.translate(translations: Map<String, String>): GeoIpLocation =
        copy(
            city = translations[city] ?: city,
            country = translations[country] ?: country,
            entryCity = translations[entryCity] ?: entryCity,
            entryCountry = translations[entryCountry] ?: entryCountry,
        )

    suspend fun connect(): Either<ConnectError, Boolean> = either {
        context.prepareVpnSafe().mapLeft(ConnectError::NotPrepared).bind()
        connectWithoutPermissionCheck().bind()
    }

    suspend fun connectWithoutPermissionCheck(): Either<ConnectError, Boolean> {
        _connectionRequests.update { it + 1 }
        return managementService.connect()
    }

    suspend fun disconnect(disconnectReason: DisconnectReason) =
        managementService.disconnect(disconnectReason)

    suspend fun reconnect(): Either<ConnectError, Boolean> {
        _connectionRequests.update { it + 1 }
        return managementService.reconnect()
    }
}
