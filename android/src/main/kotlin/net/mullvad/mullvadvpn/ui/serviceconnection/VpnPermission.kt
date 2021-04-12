package net.mullvad.mullvadvpn.ui.serviceconnection

import android.os.Messenger
import net.mullvad.mullvadvpn.ipc.Request

class VpnPermission(private val connection: Messenger) {
    fun grant(isGranted: Boolean) {
        connection.send(Request.VpnPermissionResponse(isGranted).message)
    }
}
