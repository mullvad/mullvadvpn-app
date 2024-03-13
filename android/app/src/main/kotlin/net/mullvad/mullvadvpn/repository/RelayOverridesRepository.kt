package net.mullvad.mullvadvpn.repository

import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.mapNotNull
import kotlinx.coroutines.flow.onStart
import kotlinx.coroutines.flow.stateIn
import net.mullvad.mullvadvpn.lib.ipc.MessageHandler
import net.mullvad.mullvadvpn.lib.ipc.Request
import net.mullvad.mullvadvpn.model.RelayOverride
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManager
import net.mullvad.mullvadvpn.ui.serviceconnection.settingsListener
import net.mullvad.mullvadvpn.util.callbackFlowFromNotifier
import net.mullvad.mullvadvpn.util.flatMapReadyConnectionOrDefault

class RelayOverridesRepository(
    private val serviceConnectionManager: ServiceConnectionManager,
    private val messageHandler: MessageHandler,
    dispatcher: CoroutineDispatcher = Dispatchers.IO,
) {
    fun clearAllOverrides() {
        messageHandler.trySendRequest(Request.ClearAllRelayOverrides)
    }

    val relayOverrides: StateFlow<List<RelayOverride>?> =
        serviceConnectionManager.connectionState
            .flatMapReadyConnectionOrDefault(flowOf()) { state ->
                callbackFlowFromNotifier(state.container.settingsListener.settingsNotifier)
            }
            .mapNotNull { it?.relayOverrides?.toList() }
            .onStart {
                serviceConnectionManager
                    .settingsListener()
                    ?.settingsNotifier
                    ?.latestEvent
                    ?.relayOverrides
                    ?.toList()
            }
            .stateIn(CoroutineScope(dispatcher), SharingStarted.Eagerly, null)
}
