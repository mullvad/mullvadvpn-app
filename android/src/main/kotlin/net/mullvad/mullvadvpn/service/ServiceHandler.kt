package net.mullvad.mullvadvpn.service

import android.os.Handler
import android.os.Looper
import android.os.Message
import kotlin.properties.Delegates.observable

class ServiceHandler(looper: Looper) : Handler(looper) {
    val settingsListener = SettingsListener()

    var daemon by observable<MullvadDaemon?>(null) { _, _, newDaemon ->
        settingsListener.daemon = newDaemon
    }

    override fun handleMessage(message: Message) {}
}
