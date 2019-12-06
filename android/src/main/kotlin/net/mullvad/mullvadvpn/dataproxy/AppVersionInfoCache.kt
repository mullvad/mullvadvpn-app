package net.mullvad.mullvadvpn.dataproxy

import android.content.Context
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.MainActivity
import net.mullvad.mullvadvpn.model.AppVersionInfo

class AppVersionInfoCache(val parentActivity: MainActivity) {
    companion object {
        val LEGACY_SHARED_PREFERENCES = "app_version_info_cache"
    }

    private val daemon = parentActivity.daemon
    private val setUpJob = setUp()

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
        parentActivity.getSharedPreferences(LEGACY_SHARED_PREFERENCES, Context.MODE_PRIVATE)
            .edit()
            .clear()
            .commit()
    }

    fun onDestroy() {
        setUpJob.cancel()
        tearDown()
    }

    private fun setUp() = GlobalScope.launch(Dispatchers.Default) {
        val daemon = this@AppVersionInfoCache.daemon.await()
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

    private fun tearDown() = GlobalScope.launch(Dispatchers.Default) {
        daemon.await().onAppVersionInfoChange = null
    }
}
