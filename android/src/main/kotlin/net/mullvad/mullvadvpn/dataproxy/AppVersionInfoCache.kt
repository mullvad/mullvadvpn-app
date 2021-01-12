package net.mullvad.mullvadvpn.dataproxy

import android.content.Context
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.model.AppVersionInfo
import net.mullvad.mullvadvpn.service.MullvadDaemon
import net.mullvad.mullvadvpn.service.endpoint.SettingsListener

class AppVersionInfoCache(
    val context: Context,
    val daemon: MullvadDaemon,
    val settingsListener: SettingsListener
) {
    companion object {
        val LEGACY_SHARED_PREFERENCES = "app_version_info_cache"
    }

    private val setUpJob = setUp()

    private var appVersionInfo: AppVersionInfo? = null
        set(value) {
            synchronized(this) {
                field = value
                onUpdate?.invoke()
            }
        }

    val isSupported
        get() = appVersionInfo?.supported ?: true

    val isOutdated
        get() = appVersionInfo?.suggestedUpgrade != null

    val upgradeVersion
        get() = appVersionInfo?.suggestedUpgrade

    var onUpdate: (() -> Unit)? = null
        set(value) {
            field = value
            value?.invoke()
        }

    var showBetaReleases = false
        private set(value) {
            if (field != value) {
                field = value
                onUpdate?.invoke()
            }
        }

    var version: String? = null
        private set

    init {
        settingsListener.settingsNotifier.subscribe(this) { maybeSettings ->
            maybeSettings?.let { settings ->
                showBetaReleases = settings.showBetaReleases
            }
        }
    }

    fun onCreate() {
        context.getSharedPreferences(LEGACY_SHARED_PREFERENCES, Context.MODE_PRIVATE)
            .edit()
            .clear()
            .commit()
    }

    fun onDestroy() {
        setUpJob.cancel()
        settingsListener.settingsNotifier.unsubscribe(this)
        daemon.onAppVersionInfoChange = null
    }

    private fun setUp() = GlobalScope.launch(Dispatchers.Default) {
        val currentVersion = daemon.getCurrentVersion()

        version = currentVersion

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
