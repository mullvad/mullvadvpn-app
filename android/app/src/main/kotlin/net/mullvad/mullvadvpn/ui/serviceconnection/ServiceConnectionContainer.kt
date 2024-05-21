package net.mullvad.mullvadvpn.ui.serviceconnection

import android.os.Looper
import android.os.Messenger
import android.os.RemoteException
import android.util.Log
import kotlinx.coroutines.flow.filterIsInstance
import net.mullvad.mullvadvpn.lib.ipc.DispatchingHandler
import net.mullvad.mullvadvpn.lib.ipc.Event
import net.mullvad.mullvadvpn.lib.ipc.Request
import net.mullvad.mullvadvpn.lib.ipc.extensions.trySendRequest
import org.koin.core.component.KoinComponent

// Container of classes that communicate with the service through an active connection
//
// The properties of this class can be used to send events to the service, to listen for events from
// the service and to get values received from events.
class ServiceConnectionContainer(
    val connection: Messenger,
    onServiceReady: (ServiceConnectionContainer) -> Unit,
    onVpnPermissionRequest: () -> Unit
) : KoinComponent {
    private val dispatcher =
        DispatchingHandler(Looper.getMainLooper()) { message -> Event.fromMessage(message) }

    val events = dispatcher.parsedMessages.filterIsInstance<Event>()

    val authTokenCache = AuthTokenCache(connection, dispatcher)
    val deviceDataSource = ServiceConnectionDeviceDataSource(connection, dispatcher)

    val splitTunneling = SplitTunneling(connection, dispatcher)
    val voucherRedeemer = VoucherRedeemer(connection, dispatcher)
    val vpnPermission = VpnPermission(connection, dispatcher)

    val customDns = CustomDns(connection)

    private var listenerId: Int? = null

    init {
        vpnPermission.onRequest = onVpnPermissionRequest

        dispatcher.registerHandler(Event.ListenerReady::class) { event ->
            listenerId = event.listenerId
            onServiceReady.invoke(this@ServiceConnectionContainer)
        }

        registerListener(connection)
    }

    fun trySendRequest(request: Request, logErrors: Boolean): Boolean {
        return connection.trySendRequest(request, logErrors = logErrors)
    }

    fun onDestroy() {
        unregisterListener()

        dispatcher.onDestroy()

        authTokenCache.onDestroy()
        voucherRedeemer.onDestroy()
    }

    private fun registerListener(connection: Messenger) {
        val listener = Messenger(dispatcher)
        val request = Request.RegisterListener(listener)

        try {
            connection.send(request.message)
        } catch (exception: RemoteException) {
            Log.e("mullvad", "Failed to register listener for service events", exception)
        }
    }

    private fun unregisterListener() {
        listenerId?.let { id ->
            try {
                connection.send(Request.UnregisterListener(id).message)
            } catch (exception: RemoteException) {
                Log.e("mullvad", "Failed to unregister listener for service events", exception)
            }
        }
    }
}
