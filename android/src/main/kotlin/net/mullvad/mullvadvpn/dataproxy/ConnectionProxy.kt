package net.mullvad.mullvadvpn.dataproxy

import android.content.Context
import android.content.Intent
import android.net.VpnService
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.Deferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.MainActivity
import net.mullvad.mullvadvpn.MullvadDaemon
import net.mullvad.mullvadvpn.model.TunnelState
import net.mullvad.talpid.tunnel.ActionAfterDisconnect
import net.mullvad.talpid.util.EventNotifier

val ANTICIPATED_STATE_TIMEOUT_MS = 1500L

class ConnectionProxy(val context: Context, val daemon: Deferred<MullvadDaemon>) {
    var mainActivity: MainActivity? = null

    private var activeAction: Job? = null
    private var resetAnticipatedStateJob: Job? = null

    private val attachListenerJob = attachListener()
    private val fetchInitialStateJob = fetchInitialState()

    private val initialState: TunnelState = TunnelState.Disconnected()

    var state = initialState
        private set(value) {
            field = value
            resetAnticipatedStateJob?.cancel()
            onStateChange.notify(value)
            uiState = value
        }

    var uiState = initialState
        private set(value) {
            field = value
            onUiStateChange.notify(value)
        }

    var onUiStateChange = EventNotifier(uiState)
    var onStateChange = EventNotifier(state)
    var vpnPermission = CompletableDeferred<Boolean>()

    fun connect() {
        if (anticipateConnectingState()) {
            cancelActiveAction()

            requestVpnPermission()

            activeAction = GlobalScope.launch(Dispatchers.Default) {
                if (vpnPermission.await()) {
                    daemon.await().connect()
                }
            }
        }
    }

    fun disconnect() {
        if (anticipateDisconnectingState()) {
            cancelActiveAction()
            activeAction = GlobalScope.launch(Dispatchers.Default) {
                daemon.await().disconnect()
            }
        }
    }

    fun cancelActiveAction() {
        activeAction?.cancel()
    }

    fun onDestroy() {
        onUiStateChange.unsubscribeAll()
        onStateChange.unsubscribeAll()
        attachListenerJob.cancel()
        detachListener()
        fetchInitialStateJob.cancel()
        cancelActiveAction()
    }

    private fun anticipateConnectingState(): Boolean {
        synchronized(this) {
            val currentState = uiState

            if (currentState is TunnelState.Connecting || currentState is TunnelState.Connected) {
                return false
            } else {
                scheduleToResetAnticipatedState()
                uiState = TunnelState.Connecting(null, null)
                return true
            }
        }
    }

    private fun anticipateDisconnectingState(): Boolean {
        synchronized(this) {
            val currentState = uiState

            if (currentState is TunnelState.Disconnected) {
                return false
            } else {
                scheduleToResetAnticipatedState()
                uiState = TunnelState.Disconnecting(ActionAfterDisconnect.Nothing)
                return true
            }
        }
    }

    private fun scheduleToResetAnticipatedState() {
        resetAnticipatedStateJob?.cancel()

        var currentJob: Job? = null

        val newJob = GlobalScope.launch(Dispatchers.Default) {
            delay(ANTICIPATED_STATE_TIMEOUT_MS)

            synchronized(this@ConnectionProxy) {
                if (!currentJob!!.isCancelled) {
                    uiState = state
                }
            }
        }

        currentJob = newJob
        resetAnticipatedStateJob = newJob
    }

    private fun requestVpnPermission() {
        val intent = VpnService.prepare(context)

        vpnPermission = CompletableDeferred()

        if (intent == null) {
            vpnPermission.complete(true)
        } else {
            val activity = mainActivity

            if (activity != null) {
                activity.requestVpnPermission(intent)
            } else {
                val activityIntent = Intent(context, MainActivity::class.java)
                    .putExtra(MainActivity.KEY_SHOULD_CONNECT, true)

                context.startActivity(activityIntent)
            }
        }
    }

    private fun fetchInitialState() = GlobalScope.launch(Dispatchers.Default) {
        val currentState = daemon.await().getState()

        synchronized(this) {
            if (state === initialState) {
                state = currentState
            }
        }
    }

    private fun attachListener() = GlobalScope.launch(Dispatchers.Default) {
        daemon.await().onTunnelStateChange = { newState ->
            synchronized(this) {
                state = newState
            }
        }
    }

    private fun detachListener() = GlobalScope.launch(Dispatchers.Default) {
        daemon.await().onTunnelStateChange = null
    }
}
