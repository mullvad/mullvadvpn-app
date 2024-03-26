package net.mullvad.mullvadvpn.ui.serviceconnection

import android.os.Messenger
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.launch
import mullvad_daemon.management_interface.ManagementInterface
import net.mullvad.mullvadvpn.lib.daemon.grpc.ManagementService
import net.mullvad.mullvadvpn.lib.daemon.grpc.toLocation
import net.mullvad.mullvadvpn.lib.ipc.Event
import net.mullvad.mullvadvpn.lib.ipc.EventDispatcher
import net.mullvad.mullvadvpn.lib.ipc.Request
import net.mullvad.mullvadvpn.lib.ipc.extensions.trySendRequest
import net.mullvad.mullvadvpn.model.GeoIpLocation
import net.mullvad.mullvadvpn.model.TunnelState
import net.mullvad.talpid.tunnel.ActionAfterDisconnect
import net.mullvad.talpid.util.EventNotifier
import java.net.InetAddress

const val ANTICIPATED_STATE_TIMEOUT_MS = 1500L

class ConnectionProxy(
    private val managementService: ManagementService
) {

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


