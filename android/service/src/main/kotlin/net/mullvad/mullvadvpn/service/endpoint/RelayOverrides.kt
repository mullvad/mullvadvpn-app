package net.mullvad.mullvadvpn.service.endpoint

import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.filterIsInstance
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.lib.ipc.Request

class RelayOverrides(
    private val endpoint: ServiceEndpoint,
    dispatcher: CoroutineDispatcher = Dispatchers.IO
) {
    private val scope: CoroutineScope = CoroutineScope(SupervisorJob() + dispatcher)
    private val daemon
        get() = endpoint.intermittentDaemon

    init {
        scope.launch {
            endpoint.dispatcher.parsedMessages
                .filterIsInstance<Request.SetRelayOverride>()
                .collect { daemon.await().setRelayOverride(it.override) }
        }

        scope.launch {
            endpoint.dispatcher.parsedMessages
                .filterIsInstance<Request.ClearAllRelayOverrides>()
                .collect { daemon.await().clearAllRelayOverrides() }
        }
    }

    fun onDestroy() {
        scope.cancel()
    }
}
