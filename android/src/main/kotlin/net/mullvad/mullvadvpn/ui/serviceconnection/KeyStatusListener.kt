package net.mullvad.mullvadvpn.ui.serviceconnection

import android.os.Messenger
import net.mullvad.mullvadvpn.model.KeygenEvent
import net.mullvad.mullvadvpn.service.Event.Type
import net.mullvad.mullvadvpn.service.Event.WireGuardKeyStatus
import net.mullvad.mullvadvpn.service.Request
import net.mullvad.talpid.util.EventNotifier

class KeyStatusListener(val connection: Messenger, val eventDispatcher: EventDispatcher) {
    val onKeyStatusChange = EventNotifier<KeygenEvent?>(null)

    var keyStatus by onKeyStatusChange.notifiable()
        private set

    init {
        eventDispatcher.registerHandler(Type.WireGuardKeyStatus) { event: WireGuardKeyStatus ->
            keyStatus = event.keyStatus
        }
    }

    fun generateKey() {
        connection.send(Request.WireGuardGenerateKey().message)
    }

    fun verifyKey() {
        connection.send(Request.WireGuardVerifyKey().message)
    }
}
