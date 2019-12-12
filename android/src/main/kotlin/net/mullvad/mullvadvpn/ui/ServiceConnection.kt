package net.mullvad.mullvadvpn.ui

import kotlinx.coroutines.CompletableDeferred
import net.mullvad.mullvadvpn.dataproxy.AccountCache
import net.mullvad.mullvadvpn.dataproxy.AppVersionInfoCache
import net.mullvad.mullvadvpn.dataproxy.KeyStatusListener
import net.mullvad.mullvadvpn.dataproxy.LocationInfoCache
import net.mullvad.mullvadvpn.dataproxy.RelayListListener
import net.mullvad.mullvadvpn.dataproxy.SettingsListener
import net.mullvad.mullvadvpn.dataproxy.WwwAuthTokenRetriever
import net.mullvad.mullvadvpn.service.MullvadDaemon
import net.mullvad.mullvadvpn.service.ServiceInstance
import net.mullvad.talpid.ConnectivityListener

class ServiceConnection(private val service: ServiceInstance, val mainActivity: MainActivity) {
    private val asyncDaemon = CompletableDeferred<MullvadDaemon>()
    private val asyncConnectivityListener = CompletableDeferred<ConnectivityListener>()

    val daemon = service.daemon
    val connectionProxy = service.connectionProxy
    val connectivityListener = service.connectivityListener

    val appVersionInfoCache = AppVersionInfoCache(mainActivity, daemon)
    val keyStatusListener = KeyStatusListener(daemon)
    val settingsListener = SettingsListener(asyncDaemon)
    val accountCache = AccountCache(settingsListener, daemon)
    var relayListListener = RelayListListener(asyncDaemon, settingsListener)
    val locationInfoCache = LocationInfoCache(daemon, connectivityListener, relayListListener)
    val wwwAuthTokenRetriever = WwwAuthTokenRetriever(asyncDaemon)

    init {
        asyncDaemon.complete(daemon)
        appVersionInfoCache.onCreate()

        connectionProxy.mainActivity = mainActivity
    }

    fun onDestroy() {
        asyncDaemon.cancel()

        accountCache.onDestroy()
        appVersionInfoCache.onDestroy()
        keyStatusListener.onDestroy()
        locationInfoCache.onDestroy()
        relayListListener.onDestroy()
        settingsListener.onDestroy()
    }
}
