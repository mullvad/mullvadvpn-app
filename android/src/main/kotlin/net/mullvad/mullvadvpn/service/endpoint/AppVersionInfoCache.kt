package net.mullvad.mullvadvpn.service.endpoint

import android.content.Context
import kotlin.properties.Delegates.observable
import net.mullvad.mullvadvpn.ipc.Event
import net.mullvad.mullvadvpn.model.AppVersionInfo
import net.mullvad.mullvadvpn.service.MullvadDaemon

class AppVersionInfoCache(context: Context, endpoint: ServiceEndpoint) {
    companion object {
        val LEGACY_SHARED_PREFERENCES = "app_version_info_cache"
    }

    private val daemon = endpoint.intermittentDaemon

    var appVersionInfo by observable<AppVersionInfo?>(null) { _, _, info ->
        endpoint.sendEvent(Event.AppVersionInfo(info))
    }
        private set

    var currentVersion by observable<String?>(null) { _, _, version ->
        endpoint.sendEvent(Event.CurrentVersion(version))
    }
        private set

    init {
        context.getSharedPreferences(LEGACY_SHARED_PREFERENCES, Context.MODE_PRIVATE)
            .edit()
            .clear()
            .commit()

        daemon.registerListener(this) { newDaemon ->
            newDaemon?.let { daemon ->
                initializeCurrentVersion(daemon)
                registerVersionInfoListener(daemon)
                fetchInitialVersionInfo(daemon)
            }
        }
    }

    fun onDestroy() {
        daemon.unregisterListener(this)
    }

    private fun initializeCurrentVersion(daemon: MullvadDaemon) {
        if (currentVersion == null) {
            currentVersion = daemon.getCurrentVersion()
        }
    }

    private fun registerVersionInfoListener(daemon: MullvadDaemon) {
        daemon.onAppVersionInfoChange = { newAppVersionInfo ->
            synchronized(this@AppVersionInfoCache) {
                appVersionInfo = newAppVersionInfo
            }
        }
    }

    private fun fetchInitialVersionInfo(daemon: MullvadDaemon) {
        synchronized(this@AppVersionInfoCache) {
            if (appVersionInfo == null) {
                appVersionInfo = daemon.getVersionInfo()
            }
        }
    }
}
