package net.mullvad.mullvadvpn.service.endpoint

import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.channels.ClosedReceiveChannelException
import kotlinx.coroutines.channels.actor
import kotlinx.coroutines.channels.sendBlocking
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.ipc.Event
import net.mullvad.mullvadvpn.ipc.Request
import net.mullvad.mullvadvpn.model.TunnelState
import net.mullvad.mullvadvpn.ui.MainActivity
import net.mullvad.talpid.tunnel.ActionAfterDisconnect
import net.mullvad.talpid.util.EventNotifier

val ANTICIPATED_STATE_TIMEOUT_MS = 1500L

class ConnectionProxy(val vpnPermission: VpnPermission, endpoint: ServiceEndpoint) {
    private enum class Command {
        CONNECT,
        RECONNECT,
        DISCONNECT,
    }

    private val commandChannel = spawnActor()
    private val daemon = endpoint.intermittentDaemon

    var mainActivity: MainActivity? = null

    private var resetAnticipatedStateJob: Job? = null

    private val initialState: TunnelState = TunnelState.Disconnected

    var onStateChange = EventNotifier(initialState)
    var onUiStateChange = EventNotifier(initialState)

    var state by onStateChange.notifiable()
        private set
    var uiState by onUiStateChange.notifiable()
        private set

    private val fetchInitialStateJob = fetchInitialState()

    init {
        daemon.registerListener(this) { newDaemon ->
            newDaemon?.onTunnelStateChange = { newState -> handleNewState(newState) }
        }

        onStateChange.subscribe(this) { tunnelState ->
            endpoint.sendEvent(Event.TunnelStateChange(tunnelState))
        }

        endpoint.dispatcher.apply {
            registerHandler(Request.Connect::class) { _ -> connect() }
            registerHandler(Request.Reconnect::class) { _ -> reconnect() }
            registerHandler(Request.Disconnect::class) { _ -> disconnect() }
        }
    }

    fun connect() {
        if (anticipateConnectingState()) {
            commandChannel.sendBlocking(Command.CONNECT)
        }
    }

    fun reconnect() {
        if (anticipateReconnectingState()) {
            commandChannel.sendBlocking(Command.RECONNECT)
        }
    }

    fun disconnect() {
        if (anticipateDisconnectingState()) {
            commandChannel.sendBlocking(Command.DISCONNECT)
        }
    }

    fun onDestroy() {
        commandChannel.close()

        onUiStateChange.unsubscribeAll()
        onStateChange.unsubscribeAll()

        fetchInitialStateJob.cancel()
        daemon.unregisterListener(this)
    }

    private fun spawnActor() = GlobalScope.actor<Command>(Dispatchers.Default, Channel.UNLIMITED) {
        try {
            while (true) {
                val command = channel.receive()

                when (command) {
                    Command.CONNECT -> {
                        vpnPermission.request()
                        daemon.await().connect()
                    }
                    Command.RECONNECT -> daemon.await().reconnect()
                    Command.DISCONNECT -> daemon.await().disconnect()
                }
            }
        } catch (exception: ClosedReceiveChannelException) {
            // Closed sender, so stop the actor
        }
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

    private fun fetchInitialState() = GlobalScope.launch(Dispatchers.Default) {
        val currentState = daemon.await().getState()

        synchronized(this) {
            if (state === initialState && currentState != null) {
                state = currentState
            }
        }
    }
}
