package net.mullvad.mullvadvpn.lib.repository

import net.mullvad.mullvadvpn.lib.grpc.ManagementService

class CustomTunnelsRepository(private val managementService: ManagementService) {

    suspend fun toggleCustomVpn(enable: Boolean) = managementService.togglePersonalVpn(enable)

    suspend fun getPersonalVpnStats() = managementService.getPersonalVpnStats()

    suspend fun importConfig(config: String) = managementService.importPersonalVpnConfig(config)
}
