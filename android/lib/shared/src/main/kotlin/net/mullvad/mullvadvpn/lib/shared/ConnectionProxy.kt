package net.mullvad.mullvadvpn.lib.shared

import arrow.core.Either
import arrow.core.raise.either
import arrow.core.raise.ensure
import kotlinx.coroutines.flow.combine
import mullvad_daemon.management_interface.location
import net.mullvad.mullvadvpn.lib.daemon.grpc.ManagementService
import net.mullvad.mullvadvpn.lib.model.ConnectError
import net.mullvad.mullvadvpn.lib.model.GeoIpLocation
import net.mullvad.mullvadvpn.lib.model.TunnelState
import net.mullvad.mullvadvpn.lib.model.location

class ConnectionProxy(
    private val managementService: ManagementService,
    private val translationRepository: RelayLocationTranslationRepository,
    private val vpnPermissionRepository: VpnPermissionRepository
) {
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
        copy(city = translations[city] ?: city, country = translations[country] ?: country)

    suspend fun connect(): Either<ConnectError, Boolean> = either {
        ensure(vpnPermissionRepository.hasVpnPermission()) { ConnectError.NoVpnPermission }
        managementService.connect().bind()
    }

    suspend fun connectWithoutPermissionCheck(): Either<ConnectError, Boolean> =
        managementService.connect()

    suspend fun disconnect() = managementService.disconnect()

    suspend fun reconnect() = managementService.reconnect()
}
