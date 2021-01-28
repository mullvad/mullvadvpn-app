package net.mullvad.mullvadvpn.service.endpoint

import android.content.Context
import java.io.File
import kotlin.properties.Delegates.observable
import net.mullvad.mullvadvpn.ipc.Event
import net.mullvad.mullvadvpn.ipc.Request
import net.mullvad.talpid.util.EventNotifier

// The spelling of the shared preferences location can't be changed to American English without
// either having users lose their preferences on update or implementing some migration code.
private const val SHARED_PREFERENCES = "split_tunnelling"
private const val KEY_ENABLED = "enabled"

class SplitTunneling(context: Context, endpoint: ServiceEndpoint) {
    // The spelling of the app list file name can't be changed to American English without either
    // having users lose their preferences on update or implementing some migration code.
    private val appListFile = File(context.filesDir, "split-tunnelling.txt")
    private val excludedApps = HashSet<String>()
    private val preferences = context.getSharedPreferences(SHARED_PREFERENCES, Context.MODE_PRIVATE)

    private var enabled by observable(preferences.getBoolean(KEY_ENABLED, false)) { _, _, _ ->
        enabledChanged()
    }

    val onChange = EventNotifier<List<String>?>(null)

    init {
        if (appListFile.exists()) {
            excludedApps.addAll(appListFile.readLines())
            update()
        }

        onChange.subscribe(this) { excludedApps ->
            endpoint.sendEvent(Event.SplitTunnelingUpdate(excludedApps))
        }

        endpoint.dispatcher.apply {
            registerHandler(Request.IncludeApp::class) { request ->
                excludedApps.remove(request.packageName)
                update()
            }

            registerHandler(Request.ExcludeApp::class) { request ->
                excludedApps.add(request.packageName)
                update()
            }

            registerHandler(Request.SetEnableSplitTunneling::class) { request ->
                enabled = request.enable
            }

            registerHandler(Request.PersistExcludedApps::class) { _ ->
                appListFile.writeText(excludedApps.joinToString(separator = "\n"))
            }
        }
    }

    fun onDestroy() {
        onChange.unsubscribeAll()
    }

    private fun enabledChanged() {
        preferences.edit().apply {
            putBoolean(KEY_ENABLED, enabled)
            apply()
        }

        update()
    }

    private fun update() {
        if (enabled) {
            onChange.notify(excludedApps.toList())
        } else {
            onChange.notify(null)
        }
    }
}
