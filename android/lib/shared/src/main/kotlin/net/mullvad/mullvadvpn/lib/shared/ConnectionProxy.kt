package net.mullvad.mullvadvpn.lib.shared

import arrow.core.Either
import arrow.core.raise.either
import arrow.core.raise.ensure
import net.mullvad.mullvadvpn.lib.daemon.grpc.ManagementService
import net.mullvad.mullvadvpn.lib.model.ConnectError

class ConnectionProxy(
    private val managementService: ManagementService,
    private val vpnPermissionRepository: VpnPermissionRepository
) {
    val tunnelState = managementService.tunnelState

    suspend fun connect(): Either<ConnectError, Boolean> = either {
        ensure(vpnPermissionRepository.hasVpnPermission()) { ConnectError.NoVpnPermission }
        managementService.connect().bind()
    }

    suspend fun connectWithoutPermissionCheck(): Either<ConnectError, Boolean> =
        managementService.connect()

    suspend fun disconnect() = managementService.disconnect()

    suspend fun reconnect() = managementService.reconnect()
}
