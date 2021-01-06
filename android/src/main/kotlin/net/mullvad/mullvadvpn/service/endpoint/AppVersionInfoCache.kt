package net.mullvad.mullvadvpn.service.endpoint

import kotlin.properties.Delegates.observable
import net.mullvad.mullvadvpn.ipc.Event
import net.mullvad.mullvadvpn.model.AppVersionInfo

class AppVersionInfoCache(endpoint: ServiceEndpoint) {
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
