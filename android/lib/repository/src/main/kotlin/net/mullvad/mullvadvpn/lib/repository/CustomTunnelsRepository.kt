package net.mullvad.mullvadvpn.lib.repository

import arrow.core.Either
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.withContext
import net.mullvad.mullvadvpn.lib.grpc.ManagementService
import net.mullvad.mullvadvpn.lib.model.CustomVpnConfig
import net.mullvad.mullvadvpn.lib.model.GetCustomVpnConfigError
import net.mullvad.mullvadvpn.lib.model.SetCustomVpnConfigError
import net.mullvad.mullvadvpn.lib.model.TunnelStats

class CustomTunnelsRepository(
    private val managementService: ManagementService,
    private val ioDispatcher: CoroutineDispatcher,
) {

    suspend fun setConfig(config: CustomVpnConfig): Either<SetCustomVpnConfigError, Unit> =
        withContext(ioDispatcher) { managementService.setPersonalVpnConfig(config) }

    suspend fun getPersonalVpnStats(): Either<GetCustomVpnConfigError, TunnelStats> =
        withContext(ioDispatcher) { managementService.getPersonalVpnStats() }

    suspend fun importConfig(name: String, config: String): Either<SetCustomVpnConfigError, Unit> =
        withContext(ioDispatcher) {
            managementService.importPersonalVpnConfig(name = name, config = config)
        }
}
