package net.mullvad.mullvadvpn.ui

import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.dataproxy.AccountCache
import net.mullvad.mullvadvpn.dataproxy.AppVersionInfoCache
import net.mullvad.mullvadvpn.dataproxy.ConnectionProxy
import net.mullvad.mullvadvpn.dataproxy.KeyStatusListener
import net.mullvad.mullvadvpn.dataproxy.LocationInfoCache
import net.mullvad.mullvadvpn.dataproxy.RelayListListener
import net.mullvad.mullvadvpn.dataproxy.SettingsListener
import net.mullvad.mullvadvpn.service.MullvadDaemon
import net.mullvad.talpid.ConnectivityListener

open class ServiceDependentFragment(val onNoService: OnNoService) : ServiceAwareFragment() {
    enum class OnNoService {
        GoBack, GoToLaunchScreen
    }

    lateinit var accountCache: AccountCache
        private set

    lateinit var appVersionInfoCache: AppVersionInfoCache
        private set

    lateinit var connectionProxy: ConnectionProxy
        private set

    lateinit var connectivityListener: ConnectivityListener
        private set

    lateinit var daemon: MullvadDaemon
        private set

    lateinit var keyStatusListener: KeyStatusListener
        private set

    lateinit var locationInfoCache: LocationInfoCache
        private set

    lateinit var relayListListener: RelayListListener
        private set

    lateinit var settingsListener: SettingsListener
        private set

    override fun onNewServiceConnection(serviceConnection: ServiceConnection) {
        accountCache = serviceConnection.accountCache
        appVersionInfoCache = serviceConnection.appVersionInfoCache
        connectionProxy = serviceConnection.connectionProxy
        connectivityListener = serviceConnection.connectivityListener
        daemon = serviceConnection.daemon
        keyStatusListener = serviceConnection.keyStatusListener
        locationInfoCache = serviceConnection.locationInfoCache
        relayListListener = serviceConnection.relayListListener
        settingsListener = serviceConnection.settingsListener
    }

    override fun onNoServiceConnection() {
        GlobalScope.launch(Dispatchers.Main) {
            when (onNoService) {
                OnNoService.GoBack -> parentActivity.onBackPressed()
                OnNoService.GoToLaunchScreen -> parentActivity.returnToLaunchScreen()
            }
        }
    }
}
