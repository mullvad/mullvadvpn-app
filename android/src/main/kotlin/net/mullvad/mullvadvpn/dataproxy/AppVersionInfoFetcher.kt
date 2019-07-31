package net.mullvad.mullvadvpn.dataproxy

import java.util.Calendar

import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import kotlinx.coroutines.Deferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope

import android.content.Context

import net.mullvad.mullvadvpn.MullvadDaemon

val ONE_DAY_IN_MILLISECONDS = 24L * 60L * 60L * 1000L
val ONE_MINUTE_IN_MILLISECONDS = 60L * 1000L

val KEY_CURRENT_IS_SUPPORTED = "current_is_supported"
val KEY_LAST_UPDATED = "last_updated"
val KEY_LATEST_STABLE = "latest_stable"
val KEY_LATEST = "latest"
val SHARED_PREFERENCES = "app_version_info_cache"

class AppVersionInfoFetcher(val daemon: Deferred<MullvadDaemon>, val context: Context) {
    private val preferences = context.getSharedPreferences(SHARED_PREFERENCES, Context.MODE_PRIVATE)

    private val mainLoop = run()

    fun stop() {
        mainLoop.cancel()
    }

    private fun run() = GlobalScope.launch(Dispatchers.Default) {
        while (true) {
            delay(calculateDelay())
            fetch()
        }
    }

    private fun calculateDelay(): Long {
        val now = Calendar.getInstance().timeInMillis
        val lastUpdated = preferences.getLong(KEY_LAST_UPDATED, 0)
        val delta = now - lastUpdated

        if (delta < 0 || delta >= ONE_DAY_IN_MILLISECONDS) {
            return 0
        } else {
            return ONE_DAY_IN_MILLISECONDS - delta
        }
    }

    private suspend fun fetch() {
        var now = Calendar.getInstance().timeInMillis
        var versionInfo = daemon.await().getVersionInfo()
        var attempt = 0

        while (attempt < 5 && versionInfo == null) {
            delay(ONE_MINUTE_IN_MILLISECONDS)
            now = Calendar.getInstance().timeInMillis
            versionInfo = daemon.await().getVersionInfo()
            attempt += 1
        }

        if (versionInfo != null) {
            preferences.edit().apply {
                putLong(KEY_LAST_UPDATED, now)
                putBoolean(KEY_CURRENT_IS_SUPPORTED, versionInfo.currentIsSupported)
                putString(KEY_LATEST_STABLE, versionInfo.latestStable)
                putString(KEY_LATEST, versionInfo.latest)
                commit()
            }
        }
    }
}
