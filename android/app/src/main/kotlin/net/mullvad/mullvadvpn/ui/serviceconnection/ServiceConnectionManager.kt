package net.mullvad.mullvadvpn.ui.serviceconnection

import android.content.ComponentName
import android.content.Context
import android.content.Intent
import android.os.IBinder
import android.os.Messenger
import android.util.Log
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import net.mullvad.mullvadvpn.lib.endpoint.ApiEndpointConfiguration
import net.mullvad.mullvadvpn.lib.endpoint.BuildConfig
import net.mullvad.mullvadvpn.lib.endpoint.putApiEndpointConfigurationExtra
import net.mullvad.mullvadvpn.service.MullvadVpnService
import net.mullvad.talpid.util.EventNotifier

class ServiceConnectionManager(
    private val context: Context
) {
    private val _connectionState =
        MutableStateFlow<ServiceConnectionState>(ServiceConnectionState.Disconnected)

    val connectionState = _connectionState.asStateFlow()

    // TODO: Remove after refactoring fragments to support flow.
    @Deprecated(message = "Use connectionState")
    val serviceNotifier = EventNotifier<ServiceConnectionContainer?>(null)

    private var vpnPermissionRequestHandler: (() -> Unit)? = null

    private val serviceConnection = object : android.content.ServiceConnection {
        override fun onServiceConnected(className: ComponentName, binder: IBinder) {
            Log.d("mullvad", "UI successfully connected to the service")

            notify(
                ServiceConnectionState.ConnectedNotReady(
                    ServiceConnectionContainer(
                        Messenger(binder),
                        ::handleNewServiceConnection,
                        ::handleVpnPermissionRequest
                    )
                )
            )
        }

        override fun onServiceDisconnected(className: ComponentName) {
            Log.d("mullvad", "UI lost the connection to the service")
            _connectionState.value.readyContainer()?.onDestroy()
            notify(ServiceConnectionState.Disconnected)
        }
    }

    fun bind(
        vpnPermissionRequestHandler: () -> Unit,
        apiEndpointConfiguration: ApiEndpointConfiguration?
    ) {
        this.vpnPermissionRequestHandler = vpnPermissionRequestHandler
        val intent = Intent(context, MullvadVpnService::class.java)

        if (BuildConfig.DEBUG && apiEndpointConfiguration != null) {
            intent.putApiEndpointConfigurationExtra(apiEndpointConfiguration)
        }

        context.startService(intent)
        context.bindService(intent, serviceConnection, 0)
    }

    fun unbind() {
        _connectionState.value.readyContainer()?.onDestroy()
        context.unbindService(serviceConnection)
        notify(ServiceConnectionState.Disconnected)
        vpnPermissionRequestHandler = null
    }

    fun onDestroy() {
        _connectionState.value.readyContainer()?.onDestroy()
        serviceNotifier.unsubscribeAll()
        notify(ServiceConnectionState.Disconnected)
        vpnPermissionRequestHandler = null
    }

    fun onVpnPermissionResult(isGranted: Boolean) {
        _connectionState.value.let { state ->
            if (state is ServiceConnectionState.ConnectedReady) {
                state.container.vpnPermission.grant(isGranted)
            }
        }
    }

    private fun notify(state: ServiceConnectionState) {
        _connectionState.value = state

        // TODO: Remove once `serviceNotifier` is no longer used.
        if (state is ServiceConnectionState.ConnectedReady) {
            serviceNotifier.notify(state.container)
        } else if (state is ServiceConnectionState.Disconnected) {
            serviceNotifier.notify(null)
        }
    }

    private fun handleVpnPermissionRequest() {
        vpnPermissionRequestHandler?.invoke()
    }

    private fun handleNewServiceConnection(serviceConnectionContainer: ServiceConnectionContainer) {
        notify(ServiceConnectionState.ConnectedReady(serviceConnectionContainer))
    }
}
