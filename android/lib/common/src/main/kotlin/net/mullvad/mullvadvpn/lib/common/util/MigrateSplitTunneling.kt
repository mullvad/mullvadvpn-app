package net.mullvad.mullvadvpn.lib.common.util

import android.annotation.SuppressLint
import android.content.Context
import java.io.File
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import net.mullvad.mullvadvpn.model.AppId

private const val SHARED_PREFERENCES = "split_tunnelling"
private const val KEY_ENABLED = "enabled"

val mutex = Mutex()

suspend fun migrateSplitTunneling(
    context: Context,
    enableSplitTunneling: suspend (enabled: Boolean) -> Boolean,
    excludeApps: suspend (appIds: Set<AppId>) -> Boolean
): MigrateSplitTunnelingResult {
    mutex.withLock {
        // Get from shared preferences, if not found return
        val (enabled, apps) = getOldSettings(context) ?: return MigrateSplitTunnelingResult.Failed

        // Set new settings, if failed return
        if (!enableSplitTunneling(enabled)) {
            return MigrateSplitTunnelingResult.Failed
        }
        if (!excludeApps(apps.map { AppId(it) }.toSet())) {
            return MigrateSplitTunnelingResult.Failed
        }

        // Remove old settings
        removeOldSettings(context)

        return MigrateSplitTunnelingResult.Success
    }
}

fun shouldMigrateSplitTunneling(context: Context): Boolean = getOldSettings(context) == null

private fun getOldSettings(context: Context): Pair<Boolean, Set<String>>? {
    // Get from shared preferences and appListFile
    val appListFile = File(context.filesDir, "split-tunnelling.txt")
    val preferences = context.getSharedPreferences(SHARED_PREFERENCES, Context.MODE_PRIVATE)

    return when {
        !appListFile.exists() -> return null
        !preferences.contains(KEY_ENABLED) -> return null
        else -> preferences.getBoolean(KEY_ENABLED, false) to appListFile.readLines().toSet()
    }
}

@SuppressLint("ApplySharedPref")
fun removeOldSettings(context: Context) {
    // Remove from shared preferences and app list file
    val appListFile = File(context.filesDir, "split-tunnelling.txt")
    val preferences = context.getSharedPreferences(SHARED_PREFERENCES, Context.MODE_PRIVATE)

    appListFile.delete()
    preferences.edit().remove(KEY_ENABLED).apply()
}

sealed interface MigrateSplitTunnelingResult {
    data object Success : MigrateSplitTunnelingResult

    data object Failed : MigrateSplitTunnelingResult

    data object NoOldSettingsFound : MigrateSplitTunnelingResult
}
