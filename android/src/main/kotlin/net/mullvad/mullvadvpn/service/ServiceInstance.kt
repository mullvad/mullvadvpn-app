package net.mullvad.mullvadvpn.service

import android.os.Messenger
import net.mullvad.mullvadvpn.service.endpoint.AccountCache
import net.mullvad.mullvadvpn.util.Intermittent

class ServiceInstance(
    val messenger: Messenger,
    val daemon: MullvadDaemon,
    val intermittentDaemon: Intermittent<MullvadDaemon>,
    val accountCache: AccountCache,
    val connectionProxy: ConnectionProxy,
    val customDns: CustomDns,
    val splitTunneling: SplitTunneling
) {
    fun onDestroy() {
        connectionProxy.onDestroy()
        customDns.onDestroy()
    }
}
