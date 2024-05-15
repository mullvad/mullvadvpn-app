package net.mullvad.mullvadvpn.ui.serviceconnection

import android.content.ComponentName
import android.content.Context
import android.content.Context.BIND_AUTO_CREATE
import android.content.Intent
import android.content.pm.ServiceInfo
import android.os.Build
import android.os.IBinder
import android.util.Log
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import net.mullvad.mullvadvpn.lib.endpoint.ApiEndpointConfiguration
import net.mullvad.mullvadvpn.lib.endpoint.BuildConfig
import net.mullvad.mullvadvpn.lib.endpoint.putApiEndpointConfigurationExtra
import net.mullvad.mullvadvpn.service.MullvadVpnService

class ServiceConnectionManager(private val context: Context) {
    private val _connectionState =
        MutableStateFlow<ServiceConnectionState>(ServiceConnectionState.Unbound)

    val connectionState = _connectionState.asStateFlow()

    private val serviceConnection =
        object : android.content.ServiceConnection {
            @Suppress("EmptyFunctionBlock")
            override fun onServiceConnected(name: ComponentName?, service: IBinder?) {}

            @Suppress("EmptyFunctionBlock")
            override fun onServiceDisconnected(name: ComponentName?) {}

            override fun onNullBinding(name: ComponentName?) {
                error("Received onNullBinding")
            }
        }

    fun bind(apiEndpointConfiguration: ApiEndpointConfiguration?) {
        if (_connectionState.value is ServiceConnectionState.Unbound) {
            val intent = Intent(context, MullvadVpnService::class.java)

            if (BuildConfig.DEBUG && apiEndpointConfiguration != null) {
                intent.putApiEndpointConfigurationExtra(apiEndpointConfiguration)
            }

            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.UPSIDE_DOWN_CAKE) {
                context.bindService(
                    intent,
                    serviceConnection,
                    ServiceInfo.FOREGROUND_SERVICE_TYPE_SYSTEM_EXEMPTED or BIND_AUTO_CREATE
                )
            } else {
                context.bindService(intent, serviceConnection, BIND_AUTO_CREATE)
            }
            _connectionState.value = ServiceConnectionState.Bound
        } else {
            error("Service is already bound")
        }
    }

    fun unbind() {
        if (_connectionState.value is ServiceConnectionState.Bound) {
            context.unbindService(serviceConnection)
            _connectionState.value = ServiceConnectionState.Unbound
        } else {
            error("Service is not bound")
        }
    }

    fun onDestroy() {
        try {
            unbind()
        } catch (e: java.lang.IllegalStateException) {
            Log.e("ServiceConnectionManager", "We are already unbound")
        }
    }
}
