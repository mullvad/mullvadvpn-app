package net.mullvad.mullvadvpn.service

import android.content.Context
import android.content.Intent
import android.net.VpnService
import kotlin.properties.Delegates.observable
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.channels.ClosedReceiveChannelException
import kotlinx.coroutines.channels.actor
import kotlinx.coroutines.channels.sendBlocking
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.model.TunnelState
import net.mullvad.mullvadvpn.ui.MainActivity
import net.mullvad.mullvadvpn.util.Intermittent
import net.mullvad.talpid.util.EventNotifier

class ConnectionProxy(val context: Context) {
    private enum class Command {
        CONNECT,
        RECONNECT,
        DISCONNECT,
    }

    private val availableDaemon = Intermittent<MullvadDaemon>()
    private val commandChannel = spawnActor()
    private val initialState = TunnelState.Disconnected()

    val vpnPermission = Intermittent<Boolean>()

    var onStateChange = EventNotifier<TunnelState>(initialState)

    var daemon by observable<MullvadDaemon?>(null) { _, oldDaemon, newDaemon ->
        oldDaemon?.onTunnelStateChange = null
        newDaemon?.onTunnelStateChange = { newState -> state = newState }

        availableDaemon.spawnUpdate(newDaemon)
    }

    var state by onStateChange.notifiable()
        private set

    private val fetchInitialStateJob = fetchInitialState()

    fun connect() {
        commandChannel.sendBlocking(Command.CONNECT)
    }

    fun reconnect() {
        commandChannel.sendBlocking(Command.RECONNECT)
    }

    fun disconnect() {
        commandChannel.sendBlocking(Command.DISCONNECT)
    }

    fun onDestroy() {
        daemon = null

        onStateChange.unsubscribeAll()

        fetchInitialStateJob.cancel()
        commandChannel.close()
    }

    private fun spawnActor() = GlobalScope.actor<Command>(Dispatchers.Default, Channel.UNLIMITED) {
        try {
            while (true) {
                val command = channel.receive()

                when (command) {
                    Command.CONNECT -> {
                        requestVpnPermission()
                        vpnPermission.await()
                        availableDaemon.await().connect()
                    }
                    Command.RECONNECT -> availableDaemon.await().reconnect()
                    Command.DISCONNECT -> availableDaemon.await().disconnect()
                }
            }
        } catch (exception: ClosedReceiveChannelException) {
            // Closed sender, so stop the actor
        }
    }

    private suspend fun requestVpnPermission() {
        val intent = VpnService.prepare(context)

        vpnPermission.update(null)

        if (intent == null) {
            vpnPermission.update(true)
        } else {
            val activityIntent = Intent(context, MainActivity::class.java).apply {
                addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
                addFlags(Intent.FLAG_ACTIVITY_CLEAR_TOP)
                putExtra(MainActivity.KEY_SHOULD_CONNECT, true)
            }

            context.startActivity(activityIntent)
        }
    }

    private fun fetchInitialState() = GlobalScope.launch(Dispatchers.Default) {
        val currentState = availableDaemon.await().getState()

        synchronized(this) {
            if (state === initialState && currentState != null) {
                state = currentState
            }
        }
    }
}
