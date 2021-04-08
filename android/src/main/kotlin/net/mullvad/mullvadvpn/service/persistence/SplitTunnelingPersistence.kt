package net.mullvad.mullvadvpn.service.persistence

import android.content.Context
import java.io.File
import kotlin.properties.Delegates.observable

// The spelling of the shared preferences location can't be changed to American English without
// either having users lose their preferences on update or implementing some migration code.
private const val SHARED_PREFERENCES = "split_tunnelling"
private const val KEY_ENABLED = "enabled"

class SplitTunnelingPersistence(context: Context) {
    // The spelling of the app list file name can't be changed to American English without either
    // having users lose their preferences on update or implementing some migration code.
    private val appListFile = File(context.filesDir, "split-tunnelling.txt")
    private val preferences = context.getSharedPreferences(SHARED_PREFERENCES, Context.MODE_PRIVATE)

    var enabled by observable(preferences.getBoolean(KEY_ENABLED, false)) { _, _, isEnabled ->
        preferences.edit().apply {
            putBoolean(KEY_ENABLED, isEnabled)
            apply()
        }
    }

    var excludedApps by observable(loadExcludedApps()) { _, _, excludedAppsSet ->
        appListFile.writeText(excludedAppsSet.joinToString(separator = "\n"))
    }

    private fun loadExcludedApps(): Set<String> {
        return when {
            appListFile.exists() -> appListFile.readLines().toSet()
            else -> emptySet()
        }
    }
}
