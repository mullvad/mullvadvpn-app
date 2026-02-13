package net.mullvad.mullvadvpn.app.service.migration

import android.content.Context
import java.io.File

/**
 * Migration for split tunneling apps, from Shared Preferences to Daemon.
 *
 * Previously apps where stored in Shared Preferences and injected from straight into the tunnel
 * without the knowledge of the daemon. This migration happens in conjunction with the daemon.
 *
 * See: mullvad-daemon/src/migrations/v9.rs
 */
class MigrateSplitTunneling(private val context: Context) {
    fun migrate() {
        // Get old settings, if not found return
        val enabled = getOldSettings(context) ?: return

        // Migrate enable settings to file so that the daemon can read it
        migrateSplitTunnelingEnabled(context, enabled)
    }

    private fun getOldSettings(context: Context): Boolean? {
        // Get from shared preferences and appListFile
        val appListFile = File(context.filesDir, SPLIT_TUNNELING_APPS_FILE)
        val preferences = getSharedPreferences(context)

        return if (appListFile.exists() && preferences.contains(KEY_ENABLED)) {
            preferences.getBoolean(KEY_ENABLED, false)
        } else {
            null
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
