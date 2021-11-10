package net.mullvad.mullvadvpn.ui.serviceconnection

import android.os.Messenger
import net.mullvad.mullvadvpn.ipc.Request

class ForegroundController(val messenger: Messenger) {
    fun requestForcedForeground(doForceForeground: Boolean) {
        messenger.send(Request.SetForcedForeground(doForceForeground).message)
    }
}
