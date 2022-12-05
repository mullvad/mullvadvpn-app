package net.mullvad.mullvadvpn.service.endpoint

import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.channels.ClosedReceiveChannelException
import kotlinx.coroutines.channels.actor
import kotlinx.coroutines.channels.trySendBlocking
import net.mullvad.mullvadvpn.ipc.Event
import net.mullvad.mullvadvpn.ipc.Request
import net.mullvad.mullvadvpn.model.TunnelState
import net.mullvad.talpid.util.EventNotifier

class ConnectionProxy(val vpnPermission: VpnPermission, endpoint: ServiceEndpoint) {
    private enum class Command {
        CONNECT,
        RECONNECT,
        DISCONNECT,
    }

    private val commandChannel = spawnActor()
    private val daemon = endpoint.intermittentDaemon
    private val initialState = TunnelState.Disconnected

    var onStateChange = EventNotifier<TunnelState>(initialState)

    var state by onStateChange.notifiable()
        private set

    init {
        daemon.registerListener(this) { newDaemon ->
            newDaemon?.onTunnelStateChange?.subscribe(this@ConnectionProxy) { newState ->
                state = newState
            }
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
        commandChannel.trySendBlocking(Command.CONNECT)
    }

    fun reconnect() {
        commandChannel.trySendBlocking(Command.RECONNECT)
    }

    fun disconnect() {
        commandChannel.trySendBlocking(Command.DISCONNECT)
    }

    fun onDestroy() {
        commandChannel.close()
        onStateChange.unsubscribeAll()
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
}
