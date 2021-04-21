package net.mullvad.mullvadvpn.ui.serviceconnection

import android.os.Messenger
import net.mullvad.mullvadvpn.ipc.Event
import net.mullvad.mullvadvpn.ipc.EventDispatcher
import net.mullvad.mullvadvpn.ipc.Request
import net.mullvad.mullvadvpn.model.KeygenEvent
import net.mullvad.talpid.util.EventNotifier

class KeyStatusListener(val connection: Messenger, val eventDispatcher: EventDispatcher) {
    val onKeyStatusChange = EventNotifier<KeygenEvent?>(null)

    var keyStatus by onKeyStatusChange.notifiable()
        private set

    init {
        eventDispatcher.registerHandler(Event.WireGuardKeyStatus::class) { event ->
            keyStatus = event.keyStatus
        }
    }

    fun generateKey() {
        connection.send(Request.WireGuardGenerateKey.message)
    }

    fun verifyKey() {
        connection.send(Request.WireGuardVerifyKey.message)
    }

    fun onDestroy() {
        onKeyStatusChange.unsubscribeAll()
    }
}
