package net.mullvad.mullvadvpn.service.endpoint

import android.os.Looper
import net.mullvad.mullvadvpn.ipc.DispatchingHandler
import net.mullvad.mullvadvpn.ipc.Request

class ServiceEndpoint(looper: Looper) {
    internal val dispatcher = DispatchingHandler(looper) { message ->
        Request.fromMessage(message)
    }

    fun onDestroy() {
        dispatcher.onDestroy()
    }
}
