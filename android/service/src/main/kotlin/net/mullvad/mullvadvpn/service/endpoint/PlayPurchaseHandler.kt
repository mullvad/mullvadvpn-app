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
import net.mullvad.mullvadvpn.model.PlayPurchase

class PlayPurchaseHandler(
    private val endpoint: ServiceEndpoint,
    dispatcher: CoroutineDispatcher = Dispatchers.IO
) {
    private val scope: CoroutineScope = CoroutineScope(SupervisorJob() + dispatcher)
    private val daemon
        get() = endpoint.intermittentDaemon

    init {
        scope.launch {
            endpoint.dispatcher.parsedMessages
                .filterIsInstance<Request.InitPlayPurchase>()
                .collect { initializePurchase() }
        }

        scope.launch {
            endpoint.dispatcher.parsedMessages
                .filterIsInstance<Request.VerifyPlayPurchase>()
                .collect { verifyPlayPurchase(it.playPurchase) }
        }
    }

    fun onDestroy() {
        scope.cancel()
    }

    private suspend fun initializePurchase() {
        val result = daemon.await().initPlayPurchase()
        endpoint.sendEvent(Event.PlayPurchaseInitResultEvent(result))
    }

    private suspend fun verifyPlayPurchase(playPurchase: PlayPurchase) {
        val result = daemon.await().verifyPlayPurchase(playPurchase)
        endpoint.sendEvent(Event.PlayPurchaseVerifyResultEvent(result))
    }
}
