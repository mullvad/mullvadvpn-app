package net.mullvad.mullvadvpn.ui

import net.mullvad.mullvadvpn.dataproxy.AccountCache
import net.mullvad.mullvadvpn.dataproxy.AppVersionInfoCache
import net.mullvad.mullvadvpn.dataproxy.KeyStatusListener
import net.mullvad.mullvadvpn.dataproxy.LocationInfoCache
import net.mullvad.mullvadvpn.dataproxy.RelayListListener
import net.mullvad.mullvadvpn.dataproxy.SettingsListener
import net.mullvad.mullvadvpn.service.ServiceInstance

class ServiceConnection(private val service: ServiceInstance, val mainActivity: MainActivity) {
    val daemon = service.daemon
    val connectionProxy = service.connectionProxy
    val connectivityListener = service.connectivityListener

    val keyStatusListener = KeyStatusListener(daemon)
    val settingsListener = SettingsListener(daemon)
    val appVersionInfoCache = AppVersionInfoCache(mainActivity, daemon, settingsListener)
    val accountCache = AccountCache(settingsListener, daemon)
    var relayListListener = RelayListListener(daemon, settingsListener)
    val locationInfoCache = LocationInfoCache(daemon, connectivityListener)

    init {
        appVersionInfoCache.onCreate()
        connectionProxy.mainActivity = mainActivity
    }

    fun onDestroy() {
        accountCache.onDestroy()
        appVersionInfoCache.onDestroy()
        keyStatusListener.onDestroy()
        locationInfoCache.onDestroy()
        relayListListener.onDestroy()
        settingsListener.onDestroy()
    }
}
