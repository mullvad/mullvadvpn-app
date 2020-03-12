package net.mullvad.mullvadvpn.dataproxy

import android.content.Context
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.model.AppVersionInfo
import net.mullvad.mullvadvpn.service.MullvadDaemon

class AppVersionInfoCache(
    val context: Context,
    val daemon: MullvadDaemon,
    val settingsListener: SettingsListener
) {
    companion object {
        val LEGACY_SHARED_PREFERENCES = "app_version_info_cache"
    }

    private val setUpJob = setUp()

    private val settingsListenerId = settingsListener.settingsNotifier.subscribe { settings ->
        showBetaReleases = settings.showBetaReleases ?: false
    }

    private var appVersionInfo: AppVersionInfo? = null
        set(value) {
            synchronized(this) {
                upgradeVersion = if (isStable) value?.latestStable else value?.latest

                if (value != null && upgradeVersion == version) {
                    upgradeVersion = null

                    field = AppVersionInfo(
                        value.currentIsSupported,
                        /* currentIsOutdated = */ false,
                        value.latestStable,
                        value.latest
                    )
                } else {
                    field = value
                }

                onUpdate?.invoke()
            }
        }

    var onUpdate: (() -> Unit)? = null
        set(value) {
            field = value
            value?.invoke()
        }

    var showBetaReleases = false
        private set

    val latestStable
        get() = appVersionInfo?.latestStable
    val latest
        get() = appVersionInfo?.latest
    val isSupported
        get() = appVersionInfo?.currentIsSupported ?: true
    var isOutdated = false
        get() = appVersionInfo?.currentIsOutdated ?: false

    var version: String? = null
        private set
    var isStable = true
        private set

    var upgradeVersion: String? = null
        private set

    fun onCreate() {
        context.getSharedPreferences(LEGACY_SHARED_PREFERENCES, Context.MODE_PRIVATE)
            .edit()
            .clear()
            .commit()
    }

    fun onDestroy() {
        setUpJob.cancel()
        daemon.onAppVersionInfoChange = null
    }

    private fun setUp() = GlobalScope.launch(Dispatchers.Default) {
        val currentVersion = daemon.getCurrentVersion()

        version = currentVersion
        isStable = !currentVersion.contains("-")

        daemon.onAppVersionInfoChange = { newAppVersionInfo ->
            appVersionInfo = newAppVersionInfo
        }

        synchronized(this@AppVersionInfoCache) {
            val initialVersionInfo = daemon.getVersionInfo()

            if (appVersionInfo == null) {
                appVersionInfo = initialVersionInfo
            }
        }
    }
}
