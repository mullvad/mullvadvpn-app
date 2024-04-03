package net.mullvad.mullvadvpn.repository

import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.mapNotNull
import kotlinx.coroutines.flow.onStart
import kotlinx.coroutines.flow.stateIn
import net.mullvad.mullvadvpn.lib.daemon.grpc.ManagementService
import net.mullvad.mullvadvpn.model.RelayOverride

class RelayOverridesRepository(
    private val managementService: ManagementService,
    dispatcher: CoroutineDispatcher = Dispatchers.IO,
) {
    fun clearAllOverrides() {
        //        messageHandler.trySendRequest(Request.ClearAllRelayOverrides)
    }

    val relayOverrides: StateFlow<List<RelayOverride>?> =
        managementService.settings
            .mapNotNull { it.relayOverrides }
            .onStart {
                // Get relay overrides
            }
            .stateIn(CoroutineScope(dispatcher), SharingStarted.Eagerly, null)
}
