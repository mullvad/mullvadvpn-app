package net.mullvad.mullvadvpn.ui.serviceconnection

import net.mullvad.mullvadvpn.dataproxy.AppVersionInfoCache
import net.mullvad.mullvadvpn.dataproxy.RelayListListener
import net.mullvad.mullvadvpn.service.ServiceInstance
import net.mullvad.mullvadvpn.ui.MainActivity

class ServiceConnection(private val service: ServiceInstance, val mainActivity: MainActivity) {
    val daemon = service.daemon
    val accountCache = service.accountCache
    val connectionProxy = service.connectionProxy
    val customDns = service.customDns
    val keyStatusListener = service.keyStatusListener
    val locationInfoCache = service.locationInfoCache
    val settingsListener = service.settingsListener
    val splitTunneling = service.splitTunneling

    val appVersionInfoCache = AppVersionInfoCache(mainActivity, daemon, settingsListener)
    var relayListListener = RelayListListener(daemon, settingsListener)

    init {
        appVersionInfoCache.onCreate()
        connectionProxy.mainActivity = mainActivity
    }

    fun onDestroy() {
        appVersionInfoCache.onDestroy()
        relayListListener.onDestroy()
        connectionProxy.mainActivity = null
    }
}
