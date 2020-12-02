package net.mullvad.mullvadvpn.service

import android.os.Messenger
import net.mullvad.mullvadvpn.service.endpoint.LocationInfoCache
import net.mullvad.mullvadvpn.service.endpoint.SettingsListener
import net.mullvad.mullvadvpn.util.Intermittent

class ServiceInstance(
    val messenger: Messenger,
    val daemon: MullvadDaemon,
    val intermittentDaemon: Intermittent<MullvadDaemon>,
    val connectionProxy: ConnectionProxy,
    val customDns: CustomDns,
    val locationInfoCache: LocationInfoCache,
    val settingsListener: SettingsListener,
    val splitTunneling: SplitTunneling
) {
    val accountCache = AccountCache(daemon, settingsListener)
    val keyStatusListener = KeyStatusListener(daemon)

    fun onDestroy() {
        accountCache.onDestroy()
        connectionProxy.onDestroy()
        customDns.onDestroy()
        keyStatusListener.onDestroy()
        locationInfoCache.onDestroy()
    }
}
