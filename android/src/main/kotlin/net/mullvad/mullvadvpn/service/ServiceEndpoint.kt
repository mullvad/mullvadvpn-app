package net.mullvad.mullvadvpn.service

import android.os.Looper
import android.os.Messenger
import net.mullvad.mullvadvpn.util.DispatchingHandler

class ServiceEndpoint(looper: Looper) {
    internal val dispatcher = DispatchingHandler(looper) { message ->
        Request.fromMessage(message)
    }

    val messenger = Messenger(dispatcher)

    fun onDestroy() {
        dispatcher.onDestroy()
    }
}
