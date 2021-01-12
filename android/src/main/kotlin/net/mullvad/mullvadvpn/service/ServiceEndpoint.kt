package net.mullvad.mullvadvpn.service

import android.os.Looper
import android.os.Messenger
import net.mullvad.mullvadvpn.util.DispatchingHandler
import net.mullvad.mullvadvpn.util.Intermittent

class ServiceEndpoint(looper: Looper, intermittentDaemon: Intermittent<MullvadDaemon>) {
    internal val dispatcher = DispatchingHandler(looper) { message ->
        Request.fromMessage(message)
    }

    val messenger = Messenger(dispatcher)

    val settingsListener = SettingsListener(intermittentDaemon)

    fun onDestroy() {
        dispatcher.onDestroy()
        settingsListener.onDestroy()
    }
}
