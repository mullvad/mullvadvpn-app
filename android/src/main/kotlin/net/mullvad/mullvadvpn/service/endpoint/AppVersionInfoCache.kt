package net.mullvad.mullvadvpn.service.endpoint

import android.content.Context
import kotlin.properties.Delegates.observable
import net.mullvad.mullvadvpn.ipc.Event
import net.mullvad.mullvadvpn.model.AppVersionInfo

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
    }

    fun onDestroy() {
        daemon.unregisterListener(this)
    }
}
