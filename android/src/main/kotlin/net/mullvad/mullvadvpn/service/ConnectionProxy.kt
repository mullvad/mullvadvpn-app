package net.mullvad.mullvadvpn.service

import android.content.Context
import android.content.Intent
import android.net.VpnService
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.model.TunnelState
import net.mullvad.mullvadvpn.ui.MainActivity
import net.mullvad.talpid.tunnel.ActionAfterDisconnect
import net.mullvad.talpid.util.EventNotifier

val ANTICIPATED_STATE_TIMEOUT_MS = 1500L

class ConnectionProxy(val context: Context, val daemon: MullvadDaemon) {
    var mainActivity: MainActivity? = null

    private var activeAction: Job? = null
    private var resetAnticipatedStateJob: Job? = null

    private val fetchInitialStateJob = fetchInitialState()
    private val initialState: TunnelState = TunnelState.Disconnected()

    var onStateChange = EventNotifier(initialState)
    var onUiStateChange = EventNotifier(initialState)
    var vpnPermission = CompletableDeferred<Boolean>()

    var state = initialState
        private set(value) {
            field = value
            onStateChange.notify(value)
        }

    var uiState = initialState
        private set(value) {
            field = value
            onUiStateChange.notify(value)
        }

    init {
        daemon.onTunnelStateChange = { newState ->
            synchronized(this) {
                resetAnticipatedStateJob?.cancel()
                state = newState
                uiState = newState
            }
        }
    }

    fun connect() {
        if (anticipateConnectingState()) {
            cancelActiveAction()

            requestVpnPermission()

            activeAction = GlobalScope.launch(Dispatchers.Default) {
                if (vpnPermission.await()) {
                    daemon.connect()
                }
            }
        }
    }

    fun reconnect() {
        if (anticipateReconnectingState()) {
            cancelActiveAction()
            activeAction = GlobalScope.launch(Dispatchers.Default) {
                daemon.reconnect()
            }
        }
    }

    fun disconnect() {
        if (anticipateDisconnectingState()) {
            cancelActiveAction()
            activeAction = GlobalScope.launch(Dispatchers.Default) {
                daemon.disconnect()
            }
        }
    }

    fun cancelActiveAction() {
        activeAction?.cancel()
    }

    fun onDestroy() {
        daemon.onTunnelStateChange = null

        onUiStateChange.unsubscribeAll()
        onStateChange.unsubscribeAll()

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

    private fun anticipateReconnectingState(): Boolean {
        synchronized(this) {
            val currentState = uiState

            if (currentState is TunnelState.Disconnecting &&
                currentState.actionAfterDisconnect == ActionAfterDisconnect.Reconnect
            ) {
                return false
            } else {
                scheduleToResetAnticipatedState()
                uiState = TunnelState.Disconnecting(ActionAfterDisconnect.Reconnect)
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
                val activityIntent = Intent(context, MainActivity::class.java).apply {
                    addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
                    putExtra(MainActivity.KEY_SHOULD_CONNECT, true)
                }

                uiState = state

                context.startActivity(activityIntent)
            }
        }
    }

    private fun fetchInitialState() = GlobalScope.launch(Dispatchers.Default) {
        val currentState = daemon.getState()

        synchronized(this) {
            if (state === initialState) {
                state = currentState
            }
        }
    }
}
