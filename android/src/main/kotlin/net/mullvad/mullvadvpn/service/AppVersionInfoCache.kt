package net.mullvad.mullvadvpn.service

import android.content.Context
import kotlin.properties.Delegates.observable
import net.mullvad.mullvadvpn.model.AppVersionInfo
import net.mullvad.talpid.util.EventNotifier

class AppVersionInfoCache(context: Context) {
    companion object {
        val LEGACY_SHARED_PREFERENCES = "app_version_info_cache"
    }

    val appVersionInfoNotifier = EventNotifier<AppVersionInfo?>(null)
    val currentVersionNotifier = EventNotifier<String?>(null)

    var appVersionInfo by appVersionInfoNotifier.notifiable()
        private set
    var currentVersion by currentVersionNotifier.notifiable()
        private set

    var daemon by observable<MullvadDaemon?>(null) { _, oldDaemon, newDaemon ->
        oldDaemon?.onAppVersionInfoChange = null

        if (currentVersion == null && newDaemon != null) {
            currentVersion = newDaemon.getCurrentVersion()
        }

        newDaemon?.onAppVersionInfoChange = { newAppVersionInfo ->
            synchronized(this@AppVersionInfoCache) {
                appVersionInfo = newAppVersionInfo
            }
        }

        // Load initial version info
        synchronized(this@AppVersionInfoCache) {
            if (appVersionInfo == null && newDaemon != null) {
                appVersionInfo = newDaemon.getVersionInfo()
            }
        }
    }

    init {
        context.getSharedPreferences(LEGACY_SHARED_PREFERENCES, Context.MODE_PRIVATE)
            .edit()
            .clear()
            .commit()
    }

    fun onDestroy() {
        daemon = null

        appVersionInfoNotifier.unsubscribeAll()
        currentVersionNotifier.unsubscribeAll()
    }
}
