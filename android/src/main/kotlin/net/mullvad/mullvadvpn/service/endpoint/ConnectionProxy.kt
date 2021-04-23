package net.mullvad.mullvadvpn.service.endpoint

import net.mullvad.mullvadvpn.ipc.Event
import net.mullvad.mullvadvpn.ipc.Request
import net.mullvad.mullvadvpn.model.TunnelState
import net.mullvad.mullvadvpn.service.endpoint.ConnectionProxy.Command
import net.mullvad.talpid.util.EventNotifier

class ConnectionProxy(private val vpnPermission: VpnPermission, endpoint: ServiceEndpoint) :
    Actor<Command>() {
    enum class Command {
        CONNECT,
        RECONNECT,
        DISCONNECT,
    }

    private val daemon = endpoint.intermittentDaemon

    var onStateChange = EventNotifier<TunnelState>(TunnelState.Disconnected)

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

        endpoint.dispatcher.run {
            registerHandler(Request.Connect::class) { connect() }
            registerHandler(Request.Reconnect::class) { reconnect() }
            registerHandler(Request.Disconnect::class) { disconnect() }
        }
    }

    fun connect() = sendBlocking(Command.CONNECT)

    fun reconnect() = sendBlocking(Command.RECONNECT)

    fun disconnect() = sendBlocking(Command.DISCONNECT)

    fun onDestroy() {
        closeActor()
        onStateChange.unsubscribeAll()
        daemon.unregisterListener(this)
    }

    override suspend fun onNewCommand(command: Command) = when (command) {
        Command.CONNECT -> {
            vpnPermission.request()
            daemon.await().connect()
        }
        Command.RECONNECT -> daemon.await().reconnect()
        Command.DISCONNECT -> daemon.await().disconnect()
    }
}
