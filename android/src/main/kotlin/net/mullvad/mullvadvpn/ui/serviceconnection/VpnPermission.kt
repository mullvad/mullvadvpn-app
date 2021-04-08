package net.mullvad.mullvadvpn.ui.serviceconnection

import android.os.Messenger
import net.mullvad.mullvadvpn.ipc.Request

class VpnPermission(private val connection: Messenger) {
    fun grant(permission: Boolean) {
        connection.send(Request.VpnPermissionResponse(permission).message)
    }
}
