package net.mullvad.mullvadvpn.lib.repository

import arrow.core.Either
import net.mullvad.mullvadvpn.lib.grpc.ManagementService
import net.mullvad.mullvadvpn.lib.model.CustomVpnConfig
import net.mullvad.mullvadvpn.lib.model.GetCustomVpnConfigError
import net.mullvad.mullvadvpn.lib.model.SetCustomVpnConfigError
import net.mullvad.mullvadvpn.lib.model.TunnelStats

class CustomVpnRepository(private val managementService: ManagementService) {

    suspend fun setConfig(config: CustomVpnConfig): Either<SetCustomVpnConfigError, Unit> =
        managementService.setCustomVpnConfig(config)

    suspend fun getCustomVpnStats(): Either<GetCustomVpnConfigError, TunnelStats> =
        managementService.getCustomVpnStats()

    suspend fun importConfig(name: String, config: String): Either<SetCustomVpnConfigError, Unit> =
        managementService.importCustomVpnConfig(name = name, config = config)

    suspend fun clearConfig(): Either<SetCustomVpnConfigError, Unit> =
        managementService.clearCustomVpn()
}
