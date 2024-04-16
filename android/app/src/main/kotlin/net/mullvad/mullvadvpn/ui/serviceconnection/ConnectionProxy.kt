package net.mullvad.mullvadvpn.ui.serviceconnection

import arrow.core.Either
import arrow.core.raise.either
import net.mullvad.mullvadvpn.lib.daemon.grpc.ManagementService
import net.mullvad.mullvadvpn.model.ConnectError
import net.mullvad.mullvadvpn.repository.VpnPermissionRepository

class ConnectionProxy(
    private val managementService: ManagementService,
    private val permissionRepository: VpnPermissionRepository
) {
    val tunnelState = managementService.tunnelState

    suspend fun connect(): Either<ConnectError, Unit> = either {
        if (permissionRepository.hasVpnPermission()) {
            managementService.connect().map {
                if (it) {
                    Unit
                } else {
                    raise(ConnectError.Unknown(null))
                }
            }
        } else {
            raise(ConnectError.NoVpnPermission)
        }
    }

    suspend fun disconnect() {
        managementService.disconnect()
    }

    suspend fun reconnect() {
        managementService.reconnect()
    }
}
