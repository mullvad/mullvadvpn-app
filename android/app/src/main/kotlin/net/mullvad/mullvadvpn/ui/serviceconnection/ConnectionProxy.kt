package net.mullvad.mullvadvpn.ui.serviceconnection

import net.mullvad.mullvadvpn.lib.daemon.grpc.ManagementService

class ConnectionProxy(private val managementService: ManagementService) {

    init {
        /*eventDispatcher.registerHandler(Event.TunnelStateChange::class) { event ->
            handleNewState(event.tunnelState)
        }*/
    }

    val tunnelState = managementService.tunnelState

    suspend fun connect() {
        managementService.connect()
    }

    suspend fun disconnect() {
        managementService.disconnect()
    }

    suspend fun reconnect() {
        managementService.reconnect()
    }
}
