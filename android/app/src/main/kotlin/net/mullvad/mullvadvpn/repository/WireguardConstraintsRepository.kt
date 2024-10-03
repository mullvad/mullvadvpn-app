package net.mullvad.mullvadvpn.repository

import net.mullvad.mullvadvpn.lib.daemon.grpc.ManagementService
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.Port

class WireguardConstraintsRepository(private val managementService: ManagementService) {
    suspend fun updateWireguardPort(port: Constraint<Port>) =
        managementService.setWireguardPort(port)

    suspend fun setMultihop(enabled: Boolean) = managementService.setMultihop(enabled)
}
