package net.mullvad.mullvadvpn.ui

import android.os.Bundle
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.ui.serviceconnection.AppVersionInfoCache
import net.mullvad.mullvadvpn.ui.serviceconnection.AuthTokenCache
import net.mullvad.mullvadvpn.ui.serviceconnection.ConnectionProxy
import net.mullvad.mullvadvpn.ui.serviceconnection.CustomDns
import net.mullvad.mullvadvpn.ui.serviceconnection.LocationInfoCache
import net.mullvad.mullvadvpn.ui.serviceconnection.RelayListListener
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionContainer
import net.mullvad.mullvadvpn.ui.serviceconnection.SettingsListener
import net.mullvad.mullvadvpn.ui.serviceconnection.SplitTunneling

abstract class ServiceDependentFragment(private val onNoService: OnNoService) :
    ServiceAwareFragment() {
    enum class OnNoService {
        GoBack, GoToLaunchScreen
    }

    enum class State {
        Uninitialized,
        Initialized,
        Active,
        Stopped,
        LostConnection
    }

    private var state = State.Uninitialized

    lateinit var appVersionInfoCache: AppVersionInfoCache
        private set

    lateinit var authTokenCache: AuthTokenCache
        private set

    lateinit var connectionProxy: ConnectionProxy
        private set

    lateinit var customDns: CustomDns
        private set

    lateinit var locationInfoCache: LocationInfoCache
        private set

    lateinit var relayListListener: RelayListListener
        private set

    lateinit var settingsListener: SettingsListener
        private set

    lateinit var splitTunneling: SplitTunneling
        private set

    override fun onNewServiceConnection(serviceConnectionContainer: ServiceConnectionContainer) {
        // This method is always either called first or after an `onNoServiceConnection`, so the
        // initialization of the fields doesn't have to be synchronized
        appVersionInfoCache = serviceConnectionContainer.appVersionInfoCache
        authTokenCache = serviceConnectionContainer.authTokenCache
        connectionProxy = serviceConnectionContainer.connectionProxy
        customDns = serviceConnectionContainer.customDns
        locationInfoCache = serviceConnectionContainer.locationInfoCache
        relayListListener = serviceConnectionContainer.relayListListener
        settingsListener = serviceConnectionContainer.settingsListener

        splitTunneling = serviceConnectionContainer.splitTunneling

        synchronized(this) {
            when (state) {
                State.Uninitialized -> state = State.Initialized
                State.Active -> {
                    onSafelyStop()
                    onSafelyStart()
                }
                else -> Unit
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
                else -> Unit
            }
        }
    }

    override fun onCreateView(
        inflater: LayoutInflater,
        container: ViewGroup?,
        savedInstanceState: Bundle?
    ): View {
        synchronized(this) {
            return when (state) {
                State.Initialized, State.Active, State.Stopped -> {
                    onSafelyCreateView(inflater, container, savedInstanceState)
                }
                State.Uninitialized, State.LostConnection -> {
                    inflater.inflate(R.layout.missing_service, container, false)
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
                else -> Unit
            }
        }
    }

    override fun onSaveInstanceState(instanceState: Bundle) {
        synchronized(this) {
            when (state) {
                State.Initialized, State.Stopped, State.Active -> {
                    onSafelySaveInstanceState(instanceState)
                }
                else -> Unit
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
                else -> Unit
            }
        }

        super.onStop()
    }

    override fun onDestroyView() {
        synchronized(this) {
            when (state) {
                State.Initialized, State.Stopped, State.Active -> onSafelyDestroyView()
                else -> Unit
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
