package net.mullvad.mullvadvpn.service.endpoint

import android.content.Context
import android.content.Intent
import android.net.VpnService
import net.mullvad.mullvadvpn.ipc.Event
import net.mullvad.mullvadvpn.ipc.Request
import net.mullvad.mullvadvpn.ui.MainActivity
import net.mullvad.mullvadvpn.util.Intermittent

class VpnPermission(private val context: Context, private val endpoint: ServiceEndpoint) {
    private val isGranted = Intermittent<Boolean>()

    init {
        endpoint.dispatcher.registerHandler(Request.VpnPermissionResponse::class) { request ->
            isGranted.spawnUpdate(request.isGranted)
        }
    }

    suspend fun request(): Boolean {
        val intent = VpnService.prepare(context)

        if (intent == null) {
            isGranted.update(true)
        } else {
            val activityIntent = Intent(context, MainActivity::class.java).apply {
                addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
                addFlags(Intent.FLAG_ACTIVITY_CLEAR_TOP)
                putExtra(MainActivity.KEY_SHOULD_CONNECT, true)
            }

            isGranted.update(null)

            context.startActivity(activityIntent)
            endpoint.sendEvent(Event.VpnPermissionRequest)
        }

        return isGranted.await()
    }
}
