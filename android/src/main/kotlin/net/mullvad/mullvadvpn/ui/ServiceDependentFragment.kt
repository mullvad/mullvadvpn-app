package net.mullvad.mullvadvpn.ui

import android.os.Bundle
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.dataproxy.AccountCache
import net.mullvad.mullvadvpn.dataproxy.AppVersionInfoCache
import net.mullvad.mullvadvpn.dataproxy.ConnectionProxy
import net.mullvad.mullvadvpn.dataproxy.KeyStatusListener
import net.mullvad.mullvadvpn.dataproxy.RelayListListener
import net.mullvad.mullvadvpn.dataproxy.SettingsListener
import net.mullvad.mullvadvpn.service.LocationInfoCache
import net.mullvad.mullvadvpn.service.MullvadDaemon
import net.mullvad.talpid.ConnectivityListener

abstract class ServiceDependentFragment(val onNoService: OnNoService) : ServiceAwareFragment() {
    enum class OnNoService {
        GoBack, GoToLaunchScreen
    }

    enum class State {
        Uninitialized,
        Initialized,
        Active,
        Paused,
        LostConnection,
        WaitingForReconnection,
    }

    private var state = State.Uninitialized

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
        // This method is always either called first or after an `onNoServiceConnection`, so the
        // initialization of the fields doesn't have to be synchronized
        accountCache = serviceConnection.accountCache
        appVersionInfoCache = serviceConnection.appVersionInfoCache
        connectionProxy = serviceConnection.connectionProxy
        connectivityListener = serviceConnection.connectivityListener
        daemon = serviceConnection.daemon
        keyStatusListener = serviceConnection.keyStatusListener
        locationInfoCache = serviceConnection.locationInfoCache
        relayListListener = serviceConnection.relayListListener
        settingsListener = serviceConnection.settingsListener

        synchronized(this) {
            when (state) {
                State.Uninitialized -> state = State.Initialized
                State.WaitingForReconnection -> state = State.Paused
                else -> {}
            }
        }
    }

    override fun onNoServiceConnection() {
        synchronized(this) {
            when (state) {
                State.Uninitialized -> {
                    state = State.LostConnection
                    leaveFragment()
                }
                State.Active -> {
                    state = State.LostConnection
                    leaveFragment()
                }
                State.Paused -> state = State.WaitingForReconnection
                else -> {}
            }
        }
    }

    override fun onCreateView(
        inflater: LayoutInflater,
        container: ViewGroup?,
        savedInstanceState: Bundle?
    ): View {
        synchronized(this) {
            when (state) {
                State.Initialized, State.Active, State.Paused -> {
                    return onSafelyCreateView(inflater, container, savedInstanceState)
                }
                State.Uninitialized, State.LostConnection, State.WaitingForReconnection -> {
                    return inflater.inflate(R.layout.missing_service, container, false)
                }
            }
        }
    }

    override fun onResume() {
        super.onResume()

        synchronized(this) {
            when (state) {
                State.Initialized, State.Paused -> {
                    state = State.Active
                    onSafelyResume()
                }
                State.WaitingForReconnection -> {
                    state = State.LostConnection
                    leaveFragment()
                }
                else -> {}
            }
        }
    }

    override fun onSaveInstanceState(instanceState: Bundle) {
        synchronized(this) {
            when (state) {
                State.Initialized, State.Paused, State.Active -> {
                    onSafelySaveInstanceState(instanceState)
                }
                else -> {}
            }
        }
    }

    override fun onPause() {
        synchronized(this) {
            when (state) {
                State.Initialized, State.Active -> {
                    onSafelyPause()
                    state = State.Paused
                }
                else -> {}
            }
        }

        super.onPause()
    }

    override fun onDestroyView() {
        synchronized(this) {
            when (state) {
                State.Initialized, State.Paused, State.Active -> onSafelyDestroyView()
                else -> {}
            }
        }

        super.onDestroyView()
    }

    abstract fun onSafelyCreateView(
        inflater: LayoutInflater,
        container: ViewGroup?,
        savedInstanceState: Bundle?
    ): View

    open fun onSafelyResume() {
    }

    open fun onSafelySaveInstanceState(state: Bundle) {
    }

    open fun onSafelyPause() {
    }

    open fun onSafelyDestroyView() {
    }

    private fun leaveFragment() {
        GlobalScope.launch(Dispatchers.Main) {
            when (onNoService) {
                OnNoService.GoBack -> parentActivity.onBackPressed()
                OnNoService.GoToLaunchScreen -> parentActivity.returnToLaunchScreen()
            }
        }
    }
}
