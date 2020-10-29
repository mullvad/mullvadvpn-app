package net.mullvad.mullvadvpn.service

import net.mullvad.talpid.ConnectivityListener

class ServiceInstance(
    val daemon: MullvadDaemon,
    val connectionProxy: ConnectionProxy,
    val connectivityListener: ConnectivityListener,
    val customDns: CustomDns,
    val settingsListener: SettingsListener,
    val splitTunneling: SplitTunneling
) {
    val accountCache = AccountCache(daemon, settingsListener)
    val keyStatusListener = KeyStatusListener(daemon)
    val locationInfoCache = LocationInfoCache(daemon, connectionProxy, connectivityListener)

    fun onDestroy() {
        accountCache.onDestroy()
        connectionProxy.onDestroy()
        customDns.onDestroy()
        keyStatusListener.onDestroy()
        locationInfoCache.onDestroy()
        settingsListener.onDestroy()
    }
}
