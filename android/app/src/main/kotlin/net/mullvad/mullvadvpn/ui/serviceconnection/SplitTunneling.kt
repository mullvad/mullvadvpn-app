package net.mullvad.mullvadvpn.ui.serviceconnection

import android.os.Messenger
import kotlin.properties.Delegates.observable
import net.mullvad.mullvadvpn.lib.ipc.Event
import net.mullvad.mullvadvpn.lib.ipc.EventDispatcher
import net.mullvad.mullvadvpn.lib.ipc.Request

class SplitTunneling(private val connection: Messenger, eventDispatcher: EventDispatcher) {
    private var _excludedApps by
        observable(emptySet<String>()) { _, _, apps -> excludedAppsChange.invoke(apps) }

    var enabled by observable(false) { _, _, isEnabled -> enabledChange.invoke(isEnabled) }

    var enabledChange: (enabled: Boolean) -> Unit = {}
        set(value) {
            field = value
            synchronized(this) { value.invoke(enabled) }
        }

    var excludedAppsChange: (apps: Set<String>) -> Unit = {}
        set(value) {
            field = value
            synchronized(this) { value.invoke(_excludedApps) }
        }

    init {
        eventDispatcher.registerHandler(Event.SplitTunnelingUpdate::class) { event ->
            if (event.excludedApps != null) {
                enabled = true
                _excludedApps = event.excludedApps!!.toSet()
            } else {
                enabled = false
            }
        }
    }

    fun excludeApp(appPackageName: String) =
        connection.send(Request.ExcludeApp(appPackageName).message)

    fun includeApp(appPackageName: String) =
        connection.send(Request.IncludeApp(appPackageName).message)

    fun persist() = connection.send(Request.PersistExcludedApps.message)

    fun enableSplitTunneling(isEnabled: Boolean) =
        connection.send(Request.SetEnableSplitTunneling(isEnabled).message)
}
