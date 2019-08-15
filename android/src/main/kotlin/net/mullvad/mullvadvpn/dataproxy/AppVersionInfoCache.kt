package net.mullvad.mullvadvpn.dataproxy

import kotlinx.coroutines.launch
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope

import android.content.Context
import android.content.SharedPreferences
import android.content.SharedPreferences.OnSharedPreferenceChangeListener

import net.mullvad.mullvadvpn.MainActivity

class AppVersionInfoCache(val parentActivity: MainActivity) {
    companion object {
        val KEY_CURRENT_IS_SUPPORTED = "current_is_supported"
        val KEY_LAST_UPDATED = "last_updated"
        val KEY_LATEST_STABLE = "latest_stable"
        val KEY_LATEST = "latest"
        val SHARED_PREFERENCES = "app_version_info_cache"
    }

    private val preferences: SharedPreferences
        get() = parentActivity.getSharedPreferences(SHARED_PREFERENCES, Context.MODE_PRIVATE)

    private val fetchCurrentVersionJob = fetchCurrentVersion()

    var onUpdate: (() -> Unit)? = null
        set(value) {
            field = value
            value?.invoke()
        }

    var version: String? = null
        private set
    var isStable = true
        private set

    var lastUpdated = 0L
        private set
    var isSupported = true
        private set
    var latestStable: String? = null
        private set
    var latest: String? = null
        private set

    var isLatest = true
        private set
    var upgradeVersion: String? = null
        private set

    private val listener = object : OnSharedPreferenceChangeListener {
        override fun onSharedPreferenceChanged(preferences: SharedPreferences, key: String) {
            when (key) {
                KEY_CURRENT_IS_SUPPORTED -> isSupported = preferences.getBoolean(key, isSupported)
                KEY_LAST_UPDATED -> lastUpdated = preferences.getLong(key, lastUpdated)
                KEY_LATEST_STABLE -> latestStable = preferences.getString(key, latestStable)
                KEY_LATEST -> latest = preferences.getString(key, latest)
                else -> return
            }

            updateUpgradeVersion()
        }
    }

    fun onCreate() {
        preferences.registerOnSharedPreferenceChangeListener(listener)

        lastUpdated = preferences.getLong(KEY_LAST_UPDATED, 0L)
        isSupported = preferences.getBoolean(KEY_CURRENT_IS_SUPPORTED, true)
        latestStable = preferences.getString(KEY_LATEST_STABLE, null)
        latest = preferences.getString(KEY_LATEST, null)
    }

    fun onDestroy() {
        fetchCurrentVersionJob.cancel()
        preferences.unregisterOnSharedPreferenceChangeListener(listener)
    }

    private fun fetchCurrentVersion() = GlobalScope.launch(Dispatchers.Default) {
        val currentVersion = parentActivity.daemon.await().getCurrentVersion()

        version = currentVersion
        isStable = !currentVersion.contains("-")

        updateUpgradeVersion()
    }

    private fun updateUpgradeVersion() {
        val target = if (isStable) latestStable else latest

        if (target == version || target == null) {
            isLatest = true
            upgradeVersion = null
        } else {
            isLatest = false
            upgradeVersion = target
        }

        onUpdate?.invoke()
    }
}
