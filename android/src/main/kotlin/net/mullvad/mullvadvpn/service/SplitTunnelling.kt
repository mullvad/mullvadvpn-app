package net.mullvad.mullvadvpn.service

import android.content.Context
import kotlin.properties.Delegates.observable

private const val SHARED_PREFERENCES = "split_tunnelling"
private const val KEY_ENABLED = "enabled"

class SplitTunnelling(context: Context) {
    private val excludedApps = HashSet<String>()
    private val preferences = context.getSharedPreferences(SHARED_PREFERENCES, Context.MODE_PRIVATE)

    val excludedAppList
        get() = if (enabled) {
            excludedApps.toList()
        } else {
            emptyList()
        }

    var enabled by observable(preferences.getBoolean(KEY_ENABLED, false)) { _, _, _ ->
        enabledChanged()
    }

    var onChange: ((List<String>) -> Unit)? = null

    fun isAppExcluded(appPackageName: String) = excludedApps.contains(appPackageName)

    fun excludeApp(appPackageName: String) {
        excludedApps.add(appPackageName)
        update()
    }

    fun includeApp(appPackageName: String) {
        excludedApps.remove(appPackageName)
        update()
    }

    private fun enabledChanged() {
        preferences.edit().apply {
            putBoolean(KEY_ENABLED, enabled)
            apply()
        }

        update()
    }

    private fun update() {
        onChange?.invoke(excludedAppList)
    }
}
