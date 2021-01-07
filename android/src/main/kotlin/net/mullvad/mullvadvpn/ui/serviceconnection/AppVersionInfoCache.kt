package net.mullvad.mullvadvpn.ui.serviceconnection

import android.content.Context
import kotlin.properties.Delegates.observable
import net.mullvad.mullvadvpn.ipc.DispatchingHandler
import net.mullvad.mullvadvpn.ipc.Event
import net.mullvad.mullvadvpn.model.AppVersionInfo

class AppVersionInfoCache(
    val context: Context,
    eventDispatcher: DispatchingHandler<Event>,
    val settingsListener: SettingsListener
) {
    companion object {
        val LEGACY_SHARED_PREFERENCES = "app_version_info_cache"
    }

    private var appVersionInfo by observable<AppVersionInfo?>(null) { _, _, _ ->
        onUpdate?.invoke()
    }

    val isSupported
        get() = appVersionInfo?.supported ?: true

    val isOutdated
        get() = appVersionInfo?.suggestedUpgrade != null

    val upgradeVersion
        get() = appVersionInfo?.suggestedUpgrade

    var onUpdate by observable<(() -> Unit)?>(null) { _, _, callback ->
        callback?.invoke()
    }

    var showBetaReleases by observable(false) { _, wasShowing, shouldShow ->
        if (shouldShow != wasShowing) {
            onUpdate?.invoke()
        }
    }
        private set

    var version: String? = null
        private set

    init {
        eventDispatcher.apply {
            registerHandler(Event.CurrentVersion::class) { event ->
                version = event.version
            }

            registerHandler(Event.AppVersionInfo::class) { event ->
                appVersionInfo = event.versionInfo
            }
        }

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
        settingsListener.settingsNotifier.unsubscribe(this)
        onUpdate = null
    }
}
