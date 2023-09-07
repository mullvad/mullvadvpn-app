package net.mullvad.mullvadvpn.service.endpoint

import android.content.Context
import android.content.Intent
import android.net.VpnService
import net.mullvad.mullvadvpn.lib.common.constant.MAIN_ACTIVITY_CLASS
import net.mullvad.mullvadvpn.lib.common.util.Intermittent
import net.mullvad.mullvadvpn.lib.ipc.Event
import net.mullvad.mullvadvpn.lib.ipc.Request

class VpnPermission(private val context: Context, private val endpoint: ServiceEndpoint) {
    private val isGranted = Intermittent<Boolean>()

    var waitingForResponse = false
        private set

    init {
        endpoint.dispatcher.registerHandler(Request.VpnPermissionResponse::class) { request ->
            waitingForResponse = false
            isGranted.spawnUpdate(request.isGranted)
        }
    }

    suspend fun request(): Boolean {
        val intent = VpnService.prepare(context)

        if (intent == null) {
            isGranted.update(true)
        } else {
            val activityIntent =
                Intent().apply {
                    setClassName(context.packageName, MAIN_ACTIVITY_CLASS)
                    addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
                    addFlags(Intent.FLAG_ACTIVITY_CLEAR_TOP)
                }

            isGranted.update(null)
            waitingForResponse = true

            context.startActivity(activityIntent)
            endpoint.sendEvent(Event.VpnPermissionRequest)
        }

        return isGranted.await()
    }
}
