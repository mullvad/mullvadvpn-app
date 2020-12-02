package net.mullvad.mullvadvpn.service

import android.os.Handler
import android.os.Looper
import android.os.Message
import android.os.Messenger
import kotlin.properties.Delegates.observable

class ServiceHandler(looper: Looper) : Handler(looper) {
    private val listeners = mutableListOf<Messenger>()

    val settingsListener = SettingsListener()

    var daemon by observable<MullvadDaemon?>(null) { _, _, newDaemon ->
        settingsListener.daemon = newDaemon
    }

    override fun handleMessage(message: Message) {
        val request = Request.fromMessage(message)

        when (request) {
            is Request.RegisterListener -> listeners.add(request.listener)
        }
    }
}
