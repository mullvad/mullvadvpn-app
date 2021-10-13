package net.mullvad.mullvadvpn.service.endpoint

import android.app.UiModeManager
import android.content.Context
import android.content.Context.UI_MODE_SERVICE
import android.content.Intent
import android.content.res.Configuration.UI_MODE_TYPE_TELEVISION
import android.net.VpnService
import net.mullvad.mullvadvpn.ipc.Event
import net.mullvad.mullvadvpn.ipc.Request
import net.mullvad.mullvadvpn.ui.MainActivity
import net.mullvad.mullvadvpn.ui.activities.TVActivity
import net.mullvad.mullvadvpn.util.Intermittent

class VpnPermission(private val context: Context, private val endpoint: ServiceEndpoint) {
    private val activityClass = discoverActivityClass()
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
            val activityIntent = Intent(context, activityClass).apply {
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

    private fun discoverActivityClass(): Class<out MainActivity> {
        val uiModeManager = context.getSystemService(UI_MODE_SERVICE) as UiModeManager

        return if (uiModeManager.currentModeType == UI_MODE_TYPE_TELEVISION) {
            TVActivity::class.java
        } else {
            MainActivity::class.java
        }
    }
}
