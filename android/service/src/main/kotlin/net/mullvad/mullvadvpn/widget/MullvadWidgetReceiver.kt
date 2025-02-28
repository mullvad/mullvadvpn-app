package net.mullvad.mullvadvpn.widget

import android.content.Context
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.onStart
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.lib.model.TunnelState
import net.mullvad.mullvadvpn.lib.shared.ConnectionProxy

class MullvadWidgetUpdater(
    private val context: Context,
    private val connectionProxy: ConnectionProxy,
    private val scope: CoroutineScope,
) {
    private var job: Job? = null

    fun start() {
        // Just to ensure that connection is set up since the connection won't be setup without a
        // call to the daemon
        if (job != null) {
            error("MullvadWidgetUpdater already started")
        }

        job = scope.launch { launchListenToTunnelState() }
    }

    fun stop() {
        job?.cancel(message = "MullvadWidgetUpdater stopped")
            ?: error("MullvadWidgetUpdater already stopped")
        job = null
    }

    private suspend fun launchListenToTunnelState() {
        connectionProxy.tunnelState
            .onStart { emit(TunnelState.Disconnected(null)) }
            // .debounce(TUNNEL_STATE_DEBOUNCE_MS)
            .collect { MullvadAppWidget.updateAllWidgets(context) }
    }
}
