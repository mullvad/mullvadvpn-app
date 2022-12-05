package net.mullvad.mullvadvpn.ui.serviceconnection

import android.os.DeadObjectException
import android.os.Messenger
import android.util.Log
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.ipc.Event
import net.mullvad.mullvadvpn.ipc.EventDispatcher
import net.mullvad.mullvadvpn.ipc.Request
import net.mullvad.mullvadvpn.model.TunnelState
import net.mullvad.talpid.tunnel.ActionAfterDisconnect
import net.mullvad.talpid.util.EventNotifier

val ANTICIPATED_STATE_TIMEOUT_MS = 1500L

class ConnectionProxy(private val connection: Messenger, eventDispatcher: EventDispatcher) {
    private var resetAnticipatedStateJob: Job? = null

    val onStateChange = EventNotifier<TunnelState>(TunnelState.Disconnected)
    val onUiStateChange = EventNotifier<TunnelState>(TunnelState.Disconnected)

    var state by onStateChange.notifiable()
        private set
    var uiState by onUiStateChange.notifiable()
        private set

    init {
        eventDispatcher.registerHandler(Event.TunnelStateChange::class) { event ->
            handleNewState(event.tunnelState)
        }
    }

    fun connect() {
        if (anticipateConnectingState()) {
            try {
                connection.send(Request.Connect.message)
            } catch (ex: DeadObjectException) {
                Log.e("mullvad", "Unable to send connect message over dead service connection")
            }
        }
    }

    fun disconnect() {
        if (anticipateReconnectingState()) {
            try {
                connection.send(Request.Disconnect.message)
            } catch (ex: DeadObjectException) {
                Log.e("mullvad", "Unable to send disconnect message over dead service connection")
            }
        }
    }

    fun reconnect() {
        if (anticipateDisconnectingState()) {
            try {
                connection.send(Request.Reconnect.message)
            } catch (ex: DeadObjectException) {
                Log.e("mullvad", "Unable to send reconnect message over dead service connection")
            }
        }
    }

    fun onDestroy() {
        onStateChange.unsubscribeAll()
        onUiStateChange.unsubscribeAll()
    }

    private fun handleNewState(newState: TunnelState) {
        synchronized(this) {
            resetAnticipatedStateJob?.cancel()
            state = newState
            uiState = newState
        }
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

            val willReconnect = when (currentState) {
                is TunnelState.Disconnected -> false
                is TunnelState.Disconnecting -> {
                    when (currentState.actionAfterDisconnect) {
                        ActionAfterDisconnect.Nothing -> false
                        ActionAfterDisconnect.Reconnect -> true
                        ActionAfterDisconnect.Block -> true
                    }
                }
                is TunnelState.Connecting -> true
                is TunnelState.Connected -> true
                is TunnelState.Error -> true
            }

            if (willReconnect) {
                scheduleToResetAnticipatedState()
                uiState = TunnelState.Disconnecting(ActionAfterDisconnect.Reconnect)
            }

            return willReconnect
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
}
