package net.mullvad.mullvadvpn.service

import android.content.Context
import java.io.File

class MigrateSplitTunnelingRepository(
    private val context: Context
) {
    fun migrateSplitTunneling() {
        // Get old settings, if not found return
        val enabled = getOldSettings(context) ?: return

        // Migrate enable settings to file so that the daemon can read it
        migrateSplitTunnelingEnabled(context, enabled)
    }

    private fun getOldSettings(context: Context): Boolean? {
        // Get from shared preferences and appListFile
        val appListFile = File(context.filesDir, SPLIT_TUNNELING_APPS_FILE)
        val preferences = getSharedPreferences(context)

        return when {
            !appListFile.exists() -> return null
            !preferences.contains(KEY_ENABLED) -> return null
            else -> preferences.getBoolean(KEY_ENABLED, false)
        }
    }

    private fun migrateSplitTunnelingEnabled(context: Context, enabled: Boolean) {
        val enabledFile = File(context.filesDir, SPLIT_TUNNELING_ENABLED_FILE)
        if (enabledFile.createNewFile()) {
            enabledFile.writeText(enabled.toString())
        }
    }

    private fun getSharedPreferences(context: Context) =
        context.getSharedPreferences(SHARED_PREFERENCES, Context.MODE_PRIVATE)

    companion object {
        private const val SHARED_PREFERENCES = "split_tunnelling"
        private const val KEY_ENABLED = "enabled"
        private const val SPLIT_TUNNELING_APPS_FILE = "split-tunnelling.txt"
        private const val SPLIT_TUNNELING_ENABLED_FILE = "split-tunnelling-enabled.txt"
    }
}
