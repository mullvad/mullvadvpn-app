package net.mullvad.mullvadvpn.ui.serviceconnection

import android.os.Messenger
import kotlin.properties.Delegates.observable
import net.mullvad.mullvadvpn.ipc.Event
import net.mullvad.mullvadvpn.ipc.MessageDispatcher
import net.mullvad.mullvadvpn.ipc.Request

class SplitTunneling(
    private val connection: Messenger,
    eventDispatcher: MessageDispatcher<Event>
) {
    private var excludedApps: Set<String> = emptySet()

    var enabled by observable(false) { _, wasEnabled, isEnabled ->
        if (wasEnabled != isEnabled) {
            connection.send(Request.SetEnableSplitTunneling(isEnabled).message)
        }
    }

    init {
        eventDispatcher.registerHandler(Event.SplitTunnelingUpdate::class) { event ->
            if (event.excludedApps != null) {
                enabled = true
                excludedApps = event.excludedApps.toSet()
            } else {
                enabled = false
            }
        }
    }

    fun isAppExcluded(appPackageName: String): Boolean = excludedApps.contains(appPackageName)

    fun excludeApp(appPackageName: String) =
        connection.send(Request.ExcludeApp(appPackageName).message)

    fun includeApp(appPackageName: String) =
        connection.send(Request.IncludeApp(appPackageName).message)

    fun persist() = connection.send(Request.PersistExcludedApps.message)
}
