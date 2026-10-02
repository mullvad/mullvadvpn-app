package net.mullvad.mullvadvpn.lib.repository

import arrow.core.Either
import kotlinx.coroutines.channels.BufferOverflow
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.flow.asSharedFlow
import net.mullvad.mullvadvpn.lib.grpc.ManagementService
import net.mullvad.mullvadvpn.lib.model.CustomVpnConfig
import net.mullvad.mullvadvpn.lib.model.GetCustomVpnConfigError
import net.mullvad.mullvadvpn.lib.model.SetCustomVpnConfigError
import net.mullvad.mullvadvpn.lib.model.TunnelStats

class CustomVpnRepository(private val managementService: ManagementService) {

    // Use a MutableSharedFlow so we can distinguish 3 cases:
    // 1. no emit = config not yet fetched from daemon
    // 2. null = no config set
    // 3. value = config set
    private val _configFlow: MutableSharedFlow<CustomVpnConfig?> =
        MutableSharedFlow(replay = 1, onBufferOverflow = BufferOverflow.DROP_OLDEST)

    val configFlow: SharedFlow<CustomVpnConfig?> = _configFlow.asSharedFlow()

    suspend fun fetchConfig(): Either<GetCustomVpnConfigError, CustomVpnConfig?> =
        managementService
            .getCustomVpnSettings()
            .map { it.firstOrNull() }
            .onRight { _configFlow.tryEmit(it) }
            // A failed refresh must still emit so consumers can leave their loading state. Keep
            // the last known config, or report "no config" when nothing was ever loaded.
            .onLeft { _configFlow.tryEmit(_configFlow.replayCache.lastOrNull()) }

    suspend fun setConfig(config: CustomVpnConfig): Either<SetCustomVpnConfigError, Unit> =
        managementService.setCustomVpnConfig(config).onRight { _configFlow.tryEmit(config) }

    suspend fun clearConfig(): Either<SetCustomVpnConfigError, Unit> =
        managementService.clearCustomVpn().onRight { _configFlow.tryEmit(null) }

    suspend fun importConfig(name: String, config: String): Either<SetCustomVpnConfigError, Unit> =
        managementService.importCustomVpnConfig(name = name, config = config).map { fetchConfig() }

    suspend fun getCustomVpnStats(): Either<GetCustomVpnConfigError, TunnelStats> =
        managementService.getCustomVpnStats()
}
