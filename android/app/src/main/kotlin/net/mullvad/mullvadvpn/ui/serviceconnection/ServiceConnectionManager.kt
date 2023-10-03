package net.mullvad.mullvadvpn.ui.serviceconnection

import android.content.ComponentName
import android.content.Context
import android.content.Intent
import android.os.IBinder
import android.os.Messenger
import android.util.Log
import kotlin.reflect.KClass
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.emptyFlow
import kotlinx.coroutines.flow.filterIsInstance
import kotlinx.coroutines.flow.map
import net.mullvad.mullvadvpn.lib.endpoint.ApiEndpointConfiguration
import net.mullvad.mullvadvpn.lib.endpoint.BuildConfig
import net.mullvad.mullvadvpn.lib.endpoint.putApiEndpointConfigurationExtra
import net.mullvad.mullvadvpn.lib.ipc.Event
import net.mullvad.mullvadvpn.lib.ipc.MessageHandler
import net.mullvad.mullvadvpn.lib.ipc.Request
import net.mullvad.mullvadvpn.service.MullvadVpnService
import net.mullvad.mullvadvpn.util.flatMapReadyConnectionOrDefault
import net.mullvad.talpid.util.EventNotifier

class ServiceConnectionManager(private val context: Context) : MessageHandler {
    private val _connectionState =
        MutableStateFlow<ServiceConnectionState>(ServiceConnectionState.Disconnected)

    val connectionState = _connectionState.asStateFlow()

    // TODO: Remove after refactoring fragments to support flow.
    @Deprecated(message = "Use connectionState")
    val serviceNotifier = EventNotifier<ServiceConnectionContainer?>(null)

    var isBound = false
    private var vpnPermissionRequestHandler: (() -> Unit)? = null

    private val events =
        connectionState.flatMapReadyConnectionOrDefault(emptyFlow()) { it.container.events }

    private val serviceConnection =
        object : android.content.ServiceConnection {
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
        synchronized(this) {
            if (isBound.not()) {
                this.vpnPermissionRequestHandler = vpnPermissionRequestHandler
                val intent = Intent(context, MullvadVpnService::class.java)

                if (BuildConfig.DEBUG && apiEndpointConfiguration != null) {
                    intent.putApiEndpointConfigurationExtra(apiEndpointConfiguration)
                }

                context.startService(intent)
                context.bindService(intent, serviceConnection, 0)
                isBound = true
            }
        }
    }

    fun unbind() {
        synchronized(this) {
            if (isBound) {
                _connectionState.value.readyContainer()?.onDestroy()
                context.unbindService(serviceConnection)
                notify(ServiceConnectionState.Disconnected)
                vpnPermissionRequestHandler = null
                isBound = false
            }
        }
    }

    override fun <E : Event> events(klass: KClass<E>): Flow<E> {
        return events.map { it }.filterIsInstance(klass)
    }

    override fun trySendRequest(request: Request): Boolean {
        return connectionState.value.readyContainer()?.trySendRequest(request, logErrors = false)
            ?: false
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
