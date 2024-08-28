package net.mullvad.mullvadvpn.ui.serviceconnection

import android.content.Context
import android.content.Context.BIND_AUTO_CREATE
import android.content.Intent
import android.content.pm.ServiceInfo
import android.os.Build
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import net.mullvad.mullvadvpn.service.MullvadVpnService

class ServiceConnectionManager(private val context: Context) {
    private val _connectionState =
        MutableStateFlow<ServiceConnectionState>(ServiceConnectionState.Unbound)

    val connectionState = _connectionState.asStateFlow()

    // Dummy service connection to be able to bind, all communication goes over gRPC.
    private val serviceConnection = EmptyServiceConnection()

    @Synchronized
    fun bind() {
        if (_connectionState.value is ServiceConnectionState.Unbound) {
            val intent = Intent(context, MullvadVpnService::class.java)

            // We set BIND_AUTO_CREATE so that the service is started if it is not already running
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.UPSIDE_DOWN_CAKE) {
                context.bindService(
                    intent,
                    serviceConnection,
                    ServiceInfo.FOREGROUND_SERVICE_TYPE_SYSTEM_EXEMPTED or BIND_AUTO_CREATE,
                )
            } else {
                context.bindService(intent, serviceConnection, BIND_AUTO_CREATE)
            }
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
