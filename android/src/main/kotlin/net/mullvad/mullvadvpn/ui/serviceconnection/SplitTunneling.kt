package net.mullvad.mullvadvpn.ui.serviceconnection

import android.os.Messenger
import kotlin.properties.Delegates.observable
import net.mullvad.mullvadvpn.service.Event
import net.mullvad.mullvadvpn.service.Request
import net.mullvad.mullvadvpn.util.DispatchingHandler

class SplitTunneling(val connection: Messenger, eventDispatcher: DispatchingHandler<Event>) {
    private var excludedApps = HashSet<String>()

    var enabled by observable(false) { _, wasEnabled, isEnabled ->
        if (wasEnabled != isEnabled) {
            connection.send(Request.SetEnableSplitTunneling(isEnabled).message)
        }
    }

    init {
        eventDispatcher.registerHandler(Event.SplitTunnelingUpdate::class) { event ->
            if (event.excludedApps != null) {
                enabled = true
                excludedApps = HashSet(event.excludedApps)
            } else {
                enabled = false
            }
        }
    }

    fun isAppExcluded(appPackageName: String) = excludedApps.contains(appPackageName)

    fun excludeApp(appPackageName: String) {
        connection.send(Request.ExcludeApp(appPackageName).message)
    }

    fun includeApp(appPackageName: String) {
        connection.send(Request.IncludeApp(appPackageName).message)
    }

    fun persist() {
        connection.send(Request.PersistExcludedApps().message)
    }
}
