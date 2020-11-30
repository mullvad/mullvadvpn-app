package net.mullvad.mullvadvpn.service

import android.os.Looper
import net.mullvad.mullvadvpn.util.DispatchingHandler

class ServiceEndpoint(looper: Looper) {
    internal val dispatcher = DispatchingHandler(looper) { message ->
        Request.fromMessage(message)
    }

    fun onDestroy() {
        dispatcher.onDestroy()
    }
}
