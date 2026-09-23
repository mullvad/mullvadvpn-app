package net.mullvad.mullvadvpn.receiver

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import net.mullvad.mullvadvpn.app.MullvadApplication

class AppUpdatedReceiver : BroadcastReceiver() {
    override fun onReceive(context: Context, intent: Intent) {
        if (intent.action != Intent.ACTION_MY_PACKAGE_REPLACED) return

        // Restore the tunnel to the previous state that it was before the app was updated.
        // This is needed due to the system not always binding to the VPN service after an app
        // update, even though "always-on VPN" is enabled.
        // As a bonus the app will also connect automatically if the tunnel was up before upgrading
        // even though "always-on VPN" is disabled.
        (context.applicationContext as MullvadApplication).restoreTunnel()
    }
}
