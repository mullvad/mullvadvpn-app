package net.mullvad.mullvadvpn.ui.serviceconnection

import arrow.core.Either
import arrow.core.raise.either
import net.mullvad.mullvadvpn.lib.daemon.grpc.ManagementService
import net.mullvad.mullvadvpn.lib.permission.VpnPermissionRepository
import net.mullvad.mullvadvpn.model.ConnectError

class ConnectionProxy(
    private val managementService: ManagementService,
    private val vpnPermissionRepository: VpnPermissionRepository
) {
    val tunnelState = managementService.tunnelState

    suspend fun connect(ignorePermission: Boolean = false): Either<ConnectError, Boolean> = either {
        if (ignorePermission || vpnPermissionRepository.hasVpnPermission()) {
            managementService.connect().bind()
        } else {
            raise(ConnectError.NoVpnPermission)
        }
    }

    suspend fun disconnect() = managementService.disconnect()

    suspend fun reconnect() = managementService.reconnect()
}
