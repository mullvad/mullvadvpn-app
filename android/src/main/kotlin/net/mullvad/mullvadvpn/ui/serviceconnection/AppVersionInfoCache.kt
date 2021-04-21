package net.mullvad.mullvadvpn.ui.serviceconnection

import kotlin.properties.Delegates.observable
import net.mullvad.mullvadvpn.ipc.Event
import net.mullvad.mullvadvpn.ipc.EventDispatcher
import net.mullvad.mullvadvpn.model.AppVersionInfo

class AppVersionInfoCache(
    eventDispatcher: EventDispatcher,
    private val settingsListener: SettingsListener
) {
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

    fun onDestroy() {
        settingsListener.settingsNotifier.unsubscribe(this)
        onUpdate = null
    }
}
