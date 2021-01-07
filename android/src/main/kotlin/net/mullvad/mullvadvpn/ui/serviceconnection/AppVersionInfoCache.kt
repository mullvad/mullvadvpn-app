package net.mullvad.mullvadvpn.ui.serviceconnection

import kotlin.properties.Delegates.observable
import net.mullvad.mullvadvpn.model.AppVersionInfo
import net.mullvad.mullvadvpn.service.Event

class AppVersionInfoCache(
    eventDispatcher: EventDispatcher,
    val settingsListener: SettingsListener
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
            registerHandler(Event.Type.CurrentVersion) { event: Event.CurrentVersion ->
                version = event.version
            }

            registerHandler(Event.Type.AppVersionInfo) { event: Event.AppVersionInfo ->
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
