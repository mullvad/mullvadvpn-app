package net.mullvad.mullvadvpn.service

import android.os.Handler
import android.os.Looper
import android.os.Message

class ServiceHandler(looper: Looper) : Handler(looper) {
    val settingsListener = SettingsListener()

    override fun handleMessage(message: Message) {}
}
