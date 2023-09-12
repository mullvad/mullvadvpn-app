package net.mullvad.mullvadvpn.ui.serviceconnection

import android.os.Messenger
import net.mullvad.mullvadvpn.lib.ipc.Request
import net.mullvad.mullvadvpn.lib.ipc.extensions.trySendRequest
import net.mullvad.mullvadvpn.model.DnsOptions

class CustomDns(private val connection: Messenger) {

    fun setDnsOptions(dnsOptions: DnsOptions) {
        connection.trySendRequest(Request.SetDnsOptions(dnsOptions), false)
    }
}
