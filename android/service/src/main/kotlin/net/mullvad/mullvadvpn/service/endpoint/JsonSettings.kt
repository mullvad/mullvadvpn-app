package net.mullvad.mullvadvpn.service.endpoint

import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.filterIsInstance
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.lib.ipc.Event
import net.mullvad.mullvadvpn.lib.ipc.Request

class JsonSettings(
    private val endpoint: ServiceEndpoint,
    dispatcher: CoroutineDispatcher = Dispatchers.IO
) {
    private val scope: CoroutineScope = CoroutineScope(SupervisorJob() + dispatcher)
    private val daemon
        get() = endpoint.intermittentDaemon

    init {
        scope.launch {
            endpoint.dispatcher.parsedMessages
                .filterIsInstance<Request.ApplyJsonSettings>()
                .collect { applyJsonSettings(it.json) }
        }

        scope.launch {
            endpoint.dispatcher.parsedMessages
                .filterIsInstance<Request.ExportJsonSettings>()
                .collect { exportJsonSettings() }
        }
    }

    private suspend fun applyJsonSettings(json: String) {
        val result = daemon.await().applyJsonSettings(json)
        endpoint.sendEvent(Event.ApplyJsonSettingsResult(result))
    }

    private suspend fun exportJsonSettings() {
        val json = daemon.await().exportJsonSettings()
        endpoint.sendEvent(Event.ExportJsonSettingsResult(json))
    }

    fun onDestroy() {
        scope.cancel()
    }
}
