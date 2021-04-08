package net.mullvad.mullvadvpn.service.endpoint

import android.content.Context
import android.content.Intent
import android.net.VpnService
import net.mullvad.mullvadvpn.ui.MainActivity
import net.mullvad.mullvadvpn.util.Intermittent

class VpnPermission(private val context: Context) {
    private val vpnPermission = Intermittent<Boolean>()

    suspend fun request(): Boolean {
        val intent = VpnService.prepare(context)

        if (intent == null) {
            vpnPermission.update(true)
        } else {
            val activityIntent = Intent(context, MainActivity::class.java).apply {
                addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
                addFlags(Intent.FLAG_ACTIVITY_CLEAR_TOP)
                putExtra(MainActivity.KEY_SHOULD_CONNECT, true)
            }

            vpnPermission.update(null)

            context.startActivity(activityIntent)
        }

        return vpnPermission.await()
    }

    suspend fun grant(permission: Boolean) {
        vpnPermission.update(permission)
    }
}
