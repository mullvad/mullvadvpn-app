package net.mullvad.mullvadvpn.service

import android.os.Messenger

class ServiceInstance(
    val messenger: Messenger,
    val daemon: MullvadDaemon,
    val connectionProxy: ConnectionProxy,
    val customDns: CustomDns,
    val keyStatusListener: KeyStatusListener,
    val settingsListener: SettingsListener,
    val splitTunneling: SplitTunneling
) {
    val accountCache = AccountCache(daemon, settingsListener)

    fun onDestroy() {
        accountCache.onDestroy()
        connectionProxy.onDestroy()
        customDns.onDestroy()
        keyStatusListener.onDestroy()
    }
}
