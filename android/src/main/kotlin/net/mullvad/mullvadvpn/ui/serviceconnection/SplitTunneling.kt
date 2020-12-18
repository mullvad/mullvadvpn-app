package net.mullvad.mullvadvpn.ui.serviceconnection

import android.os.Messenger
import kotlin.properties.Delegates.observable
import net.mullvad.mullvadvpn.service.Event.SplitTunnelingUpdate
import net.mullvad.mullvadvpn.service.Event.Type
import net.mullvad.mullvadvpn.service.Request

class SplitTunneling(val connection: Messenger, eventDispatcher: EventDispatcher) {
    private var excludedApps = HashSet<String>()

    var enabled by observable(false) { _, wasEnabled, isEnabled ->
        if (wasEnabled != isEnabled) {
            connection.send(Request.SetEnableSplitTunneling(isEnabled).message)
        }
    }

    init {
        eventDispatcher.registerHandler(Type.SplitTunnelingUpdate) { event: SplitTunnelingUpdate ->
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
