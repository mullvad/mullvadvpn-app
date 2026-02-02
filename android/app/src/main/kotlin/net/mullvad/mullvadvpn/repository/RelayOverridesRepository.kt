package net.mullvad.mullvadvpn.repository

import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.mapNotNull
import kotlinx.coroutines.flow.stateIn
import net.mullvad.mullvadvpn.lib.grpc.ManagementService
import net.mullvad.mullvadvpn.lib.model.RelayOverride

class RelayOverridesRepository(
    private val managementService: ManagementService,
    dispatcher: CoroutineDispatcher = Dispatchers.IO,
) {
    suspend fun clearAllOverrides() = managementService.clearAllRelayOverrides()

    suspend fun applySettingsPatch(json: String) = managementService.applySettingsPatch(json)

    val relayOverrides: StateFlow<List<RelayOverride>?> =
        managementService.settings
            .mapNotNull { it.relayOverrides }
            .stateIn(CoroutineScope(dispatcher), SharingStarted.Eagerly, null)
}
