package net.mullvad.mullvadvpn.serviceconnection

import android.content.Context
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import net.mullvad.mullvadvpn.lib.common.serviceconnection.EmptyServiceConnection
import net.mullvad.mullvadvpn.lib.common.serviceconnection.bindVpnService

class ServiceConnectionManager(private val context: Context) {
    private val _connectionState =
        MutableStateFlow<ServiceConnectionState>(ServiceConnectionState.Unbound)

    val connectionState = _connectionState.asStateFlow()

    // Dummy service connection to be able to bind, all communication goes over gRPC.
    private val serviceConnection = EmptyServiceConnection()

    @Synchronized
    fun bind() {
        if (_connectionState.value is ServiceConnectionState.Unbound) {
            context.bindVpnService(serviceConnection)
            _connectionState.value = ServiceConnectionState.Bound
        } else {
            error("Service is already bound")
        }
    }

    @Synchronized
    fun unbind() {
        if (_connectionState.value is ServiceConnectionState.Bound) {
            context.unbindService(serviceConnection)
            _connectionState.value = ServiceConnectionState.Unbound
        } else {
            error("Service is not bound")
        }
    }
}
