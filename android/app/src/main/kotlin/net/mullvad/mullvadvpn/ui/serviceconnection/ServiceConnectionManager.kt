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
            override fun onServiceConnected(className: ComponentName, binder: IBinder) {
                Log.d("ServiceConnectionManager", "Service is bound")
                // Start gRPC
            }

            override fun onServiceDisconnected(className: ComponentName) {
                Log.d("ServiceConnectionManager", "Service is unbound")
                // Stop gRPC
            }

            override fun onBindingDied(name: ComponentName?) {
                Log.d("ServiceConnectionManager", "Service is onBindingDied")
            }

            override fun onNullBinding(name: ComponentName?) {
                Log.d("ServiceConnectionManager", "onNullBinding")
                throw RuntimeException("Received onNullBinding, why u do this to me?")
            }
        }

    fun bind(apiEndpointConfiguration: ApiEndpointConfiguration?) {
        if (_connectionState.value is ServiceConnectionState.Unbound) {
            //            this.vpnPermissionRequestHandler = vpnPermissionRequestHandler
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
            throw IllegalStateException("Service is already bound")
        }
    }

    fun unbind() {
        if (_connectionState.value is ServiceConnectionState.Bound) {
            context.unbindService(serviceConnection)
            _connectionState.value = ServiceConnectionState.Unbound
        } else {
            throw IllegalStateException("Service is already bound")
        }
    }

    fun onDestroy() {
        unbind()
    }

    //    fun onVpnPermissionResult(isGranted: Boolean) {
    //        _connectionState.value.let { state ->
    //            if (state is ServiceConnectionState.ConnectedReady) {
    //                state.container.vpnPermission.grant(isGranted)
    //            }
    //        }
    //    }

    //    private fun handleVpnPermissionRequest() {
    //        vpnPermissionRequestHandler?.invoke()
    //    }
}
