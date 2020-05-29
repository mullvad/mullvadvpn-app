package net.mullvad.mullvadvpn.service

import net.mullvad.talpid.ConnectivityListener

class ServiceInstance(
    val daemon: MullvadDaemon,
    val connectionProxy: ConnectionProxy,
    val connectivityListener: ConnectivityListener,
    val settingsListener: SettingsListener
) {
    val accountCache = AccountCache(daemon, settingsListener)
    val locationInfoCache = LocationInfoCache(daemon, connectionProxy, connectivityListener)

    fun onDestroy() {
        accountCache.onDestroy()
        connectionProxy.onDestroy()
        locationInfoCache.onDestroy()
        settingsListener.onDestroy()
    }
}
