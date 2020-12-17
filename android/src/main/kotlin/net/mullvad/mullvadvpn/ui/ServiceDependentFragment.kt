package net.mullvad.mullvadvpn.ui

import android.os.Bundle
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.dataproxy.AppVersionInfoCache
import net.mullvad.mullvadvpn.dataproxy.RelayListListener
import net.mullvad.mullvadvpn.service.ConnectionProxy
import net.mullvad.mullvadvpn.service.CustomDns
import net.mullvad.mullvadvpn.service.MullvadDaemon
import net.mullvad.mullvadvpn.service.SplitTunneling
import net.mullvad.mullvadvpn.ui.serviceconnection.AccountCache
import net.mullvad.mullvadvpn.ui.serviceconnection.KeyStatusListener
import net.mullvad.mullvadvpn.ui.serviceconnection.LocationInfoCache
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnection
import net.mullvad.mullvadvpn.ui.serviceconnection.SettingsListener

abstract class ServiceDependentFragment(val onNoService: OnNoService) : ServiceAwareFragment() {
    enum class OnNoService {
        GoBack, GoToLaunchScreen
    }

    enum class State {
        Uninitialized,
        Initialized,
        Active,
        Stopped,
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

    lateinit var customDns: CustomDns
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

    lateinit var splitTunneling: SplitTunneling
        private set

    override fun onNewServiceConnection(serviceConnection: ServiceConnection) {
        // This method is always either called first or after an `onNoServiceConnection`, so the
        // initialization of the fields doesn't have to be synchronized
        accountCache = serviceConnection.accountCache
        appVersionInfoCache = serviceConnection.appVersionInfoCache
        connectionProxy = serviceConnection.connectionProxy
        customDns = serviceConnection.customDns
        daemon = serviceConnection.daemon
        keyStatusListener = serviceConnection.keyStatusListener
        locationInfoCache = serviceConnection.locationInfoCache
        relayListListener = serviceConnection.relayListListener
        settingsListener = serviceConnection.settingsListener
        splitTunneling = serviceConnection.splitTunneling

        synchronized(this) {
            when (state) {
                State.Uninitialized -> state = State.Initialized
                State.WaitingForReconnection -> state = State.Stopped
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
                State.Stopped -> state = State.WaitingForReconnection
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
                State.Initialized, State.Active, State.Stopped -> {
                    return onSafelyCreateView(inflater, container, savedInstanceState)
                }
                State.Uninitialized, State.LostConnection, State.WaitingForReconnection -> {
                    return inflater.inflate(R.layout.missing_service, container, false)
                }
            }
        }
    }

    override fun onStart() {
        super.onStart()

        synchronized(this) {
            when (state) {
                State.Initialized, State.Stopped -> {
                    state = State.Active
                    onSafelyStart()
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
                State.Initialized, State.Stopped, State.Active -> {
                    onSafelySaveInstanceState(instanceState)
                }
                else -> {}
            }
        }
    }

    override fun onStop() {
        synchronized(this) {
            when (state) {
                State.Initialized, State.Active -> {
                    onSafelyStop()
                    state = State.Stopped
                }
                else -> {}
            }
        }

        super.onStop()
    }

    override fun onDestroyView() {
        synchronized(this) {
            when (state) {
                State.Initialized, State.Stopped, State.Active -> onSafelyDestroyView()
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

    open fun onSafelyStart() {
    }

    open fun onSafelySaveInstanceState(state: Bundle) {
    }

    open fun onSafelyStop() {
    }

    open fun onSafelyDestroyView() {
    }

    private fun leaveFragment() {
        jobTracker.newUiJob("leaveFragment") {
            when (onNoService) {
                OnNoService.GoBack -> parentActivity.onBackPressed()
                OnNoService.GoToLaunchScreen -> parentActivity.returnToLaunchScreen()
            }
        }
    }
}
