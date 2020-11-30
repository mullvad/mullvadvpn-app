package net.mullvad.mullvadvpn.service.endpoint

import android.os.Looper
import android.os.Messenger
import net.mullvad.mullvadvpn.ipc.DispatchingHandler
import net.mullvad.mullvadvpn.ipc.Request

class ServiceEndpoint(looper: Looper) {
    internal val dispatcher = DispatchingHandler(looper) { message ->
        Request.fromMessage(message)
    }

    val messenger = Messenger(dispatcher)

    fun onDestroy() {
        dispatcher.onDestroy()
    }
}
