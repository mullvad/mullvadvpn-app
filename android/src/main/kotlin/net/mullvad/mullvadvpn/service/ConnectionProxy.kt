package net.mullvad.mullvadvpn.service

import android.content.Context
import android.content.Intent
import android.net.VpnService
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.channels.ClosedReceiveChannelException
import kotlinx.coroutines.channels.actor
import kotlinx.coroutines.channels.sendBlocking
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.model.TunnelState
import net.mullvad.mullvadvpn.ui.MainActivity
import net.mullvad.mullvadvpn.util.Intermittent
import net.mullvad.talpid.tunnel.ActionAfterDisconnect
import net.mullvad.talpid.util.EventNotifier

val ANTICIPATED_STATE_TIMEOUT_MS = 1500L

class ConnectionProxy(val context: Context, val daemon: MullvadDaemon) {
    private enum class Command {
        CONNECT,
        RECONNECT,
        DISCONNECT,
    }

    private val commandChannel = spawnActor()

    var mainActivity: MainActivity? = null

    private var resetAnticipatedStateJob: Job? = null

    private val initialState: TunnelState = TunnelState.Disconnected

    val vpnPermission = Intermittent<Boolean>()

    var onStateChange = EventNotifier(initialState)
    var onUiStateChange = EventNotifier(initialState)

    var state by onStateChange.notifiable()
        private set
    var uiState by onUiStateChange.notifiable()
        private set

    private val fetchInitialStateJob = fetchInitialState()

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
        daemon.onTunnelStateChange = null

        onUiStateChange.unsubscribeAll()
        onStateChange.unsubscribeAll()

        fetchInitialStateJob.cancel()
    }

    private fun spawnActor() = GlobalScope.actor<Command>(Dispatchers.Default, Channel.UNLIMITED) {
        try {
            while (true) {
                val command = channel.receive()

                when (command) {
                    Command.CONNECT -> {
                        requestVpnPermission()
                        vpnPermission.await()
                        daemon.connect()
                    }
                    Command.RECONNECT -> daemon.reconnect()
                    Command.DISCONNECT -> daemon.disconnect()
                }
            }
        } catch (exception: ClosedReceiveChannelException) {
            // Closed sender, so stop the actor
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

    private suspend fun requestVpnPermission() {
        val intent = VpnService.prepare(context)

        vpnPermission.update(null)

        if (intent == null) {
            vpnPermission.update(true)
        } else {
            val activity = mainActivity

            if (activity != null) {
                activity.requestVpnPermission(intent)
            } else {
                val activityIntent = Intent(context, MainActivity::class.java).apply {
                    addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
                    addFlags(Intent.FLAG_ACTIVITY_CLEAR_TOP)
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
            if (state === initialState && currentState != null) {
                state = currentState
            }
        }
    }
}
