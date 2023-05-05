package net.mullvad.mullvadvpn.ui.fragment

import android.content.Intent
import android.net.Uri
import android.os.Bundle
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import androidx.core.content.ContextCompat
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.lifecycleScope
import androidx.lifecycle.repeatOnLifecycle
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.model.TunnelState
import net.mullvad.mullvadvpn.repository.AccountRepository
import net.mullvad.mullvadvpn.ui.ConnectActionButton
import net.mullvad.mullvadvpn.ui.ConnectionStatus
import net.mullvad.mullvadvpn.ui.LocationInfo
import net.mullvad.mullvadvpn.ui.NavigationBarPainter
import net.mullvad.mullvadvpn.ui.extension.requireMainActivity
import net.mullvad.mullvadvpn.ui.notification.AccountExpiryNotification
import net.mullvad.mullvadvpn.ui.notification.TunnelStateNotification
import net.mullvad.mullvadvpn.ui.notification.VersionInfoNotification
import net.mullvad.mullvadvpn.ui.paintNavigationBar
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManager
import net.mullvad.mullvadvpn.ui.serviceconnection.authTokenCache
import net.mullvad.mullvadvpn.ui.serviceconnection.connectionProxy
import net.mullvad.mullvadvpn.ui.widget.HeaderBar
import net.mullvad.mullvadvpn.ui.widget.NotificationBanner
import net.mullvad.mullvadvpn.ui.widget.SwitchLocationButton
import net.mullvad.mullvadvpn.util.JobTracker
import net.mullvad.mullvadvpn.viewmodel.ConnectViewModel
import net.mullvad.talpid.tunnel.ErrorStateCause
import org.koin.android.ext.android.inject
import org.koin.androidx.viewmodel.ext.android.viewModel

class ConnectFragment : BaseFragment(), NavigationBarPainter {

    // Injected dependencies
    private val accountRepository: AccountRepository by inject()
    private val accountExpiryNotification: AccountExpiryNotification by inject()
    private val connectViewModel: ConnectViewModel by viewModel()
    private val serviceConnectionManager: ServiceConnectionManager by inject()
    private val tunnelStateNotification: TunnelStateNotification by inject()
    private val versionInfoNotification: VersionInfoNotification by inject()

    private lateinit var actionButton: ConnectActionButton
    private lateinit var switchLocationButton: SwitchLocationButton
    private lateinit var headerBar: HeaderBar
    private lateinit var notificationBanner: NotificationBanner
    private lateinit var status: ConnectionStatus
    private lateinit var locationInfo: LocationInfo

    @Deprecated("Refactor code to instead rely on Lifecycle.") private val jobTracker = JobTracker()

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        lifecycleScope.launchUiSubscriptionsOnResume()
    }

    override fun onCreateView(
        inflater: LayoutInflater,
        container: ViewGroup?,
        savedInstanceState: Bundle?
    ): View? {
        val view = inflater.inflate(R.layout.connect, container, false)

        headerBar =
            view.findViewById<HeaderBar>(R.id.header_bar).apply {
                tunnelState = connectViewModel.tunnelUiState()
            }

        accountExpiryNotification.onClick = {
            serviceConnectionManager.authTokenCache()?.fetchAuthToken()?.let { token ->
                val url = getString(R.string.account_url)
                val ready = Uri.parse("$url?token=$token")
                requireContext().startActivity(Intent(Intent.ACTION_VIEW, ready))
            }
        }

        notificationBanner =
            view.findViewById<NotificationBanner>(R.id.notification_banner).apply {
                notifications.apply {
                    // NOTE: The order of below notifications is significant.
                    register(tunnelStateNotification)
                    register(versionInfoNotification)
                    register(accountExpiryNotification)
                }
            }

        status = ConnectionStatus(view, requireMainActivity())

        locationInfo =
            LocationInfo(view, requireContext()) { connectViewModel.toggleTunnelInfoExpansion() }

        actionButton = ConnectActionButton(view)

        actionButton.apply {
            onConnect = { serviceConnectionManager.connectionProxy()?.connect() }
            onCancel = { serviceConnectionManager.connectionProxy()?.disconnect() }
            onReconnect = { serviceConnectionManager.connectionProxy()?.reconnect() }
            onDisconnect = { serviceConnectionManager.connectionProxy()?.disconnect() }
        }

        switchLocationButton =
            view.findViewById<SwitchLocationButton>(R.id.switch_location).apply {
                onClick = { openSwitchLocationScreen() }
            }

        return view
    }

    override fun onStart() {
        super.onStart()
        notificationBanner.onResume()
    }

    override fun onResume() {
        super.onResume()
        paintNavigationBar(ContextCompat.getColor(requireContext(), R.color.blue))
    }

    private fun CoroutineScope.launchUiSubscriptionsOnResume() = launch {
        repeatOnLifecycle(Lifecycle.State.RESUMED) {
            launchViewModelSubscription()
            launchAccountExpirySubscription()
        }
    }

    private fun CoroutineScope.launchViewModelSubscription() = launch {
        connectViewModel.uiState.collect { uiState ->
            locationInfo.location = uiState.location
            switchLocationButton.location = uiState.relayLocation
            uiState.versionInfo?.let {
                versionInfoNotification.updateVersionInfo(uiState.versionInfo)
            }
            tunnelStateNotification.updateTunnelState(uiState.tunnelUiState)
            updateTunnelState(uiState.tunnelUiState, uiState.tunnelRealState)
            locationInfo.isTunnelInfoExpanded = uiState.isTunnelInfoExpanded
        }
    }

    private fun CoroutineScope.launchAccountExpirySubscription() = launch {
        accountRepository.accountExpiryState.collect {
            accountExpiryNotification.updateAccountExpiry(it.date())
        }
    }

    private fun updateTunnelState(uiState: TunnelState, realState: TunnelState) {
        locationInfo.state = realState
        headerBar.tunnelState = realState
        status.setState(realState)

        actionButton.tunnelState = uiState
        switchLocationButton.tunnelState = uiState

        if (realState.isTunnelErrorStateDueToExpiredAccount()) {
            openOutOfTimeScreen()
        }
    }

    private fun openSwitchLocationScreen() {
        parentFragmentManager.beginTransaction().apply {
            setCustomAnimations(
                R.anim.fragment_enter_from_bottom,
                R.anim.do_nothing,
                R.anim.do_nothing,
                R.anim.fragment_exit_to_bottom
            )
            replace(R.id.main_fragment, SelectLocationFragment())
            addToBackStack(null)
            commitAllowingStateLoss()
        }
    }

    private fun openOutOfTimeScreen() {
        jobTracker.newUiJob("openOutOfTimeScreen") {
            parentFragmentManager.beginTransaction().apply {
                replace(R.id.main_fragment, OutOfTimeFragment())
                commitAllowingStateLoss()
            }
        }
    }

    private fun TunnelState.isTunnelErrorStateDueToExpiredAccount(): Boolean {
        return ((this as? TunnelState.Error)?.errorState?.cause as? ErrorStateCause.AuthFailed)
            ?.isCausedByExpiredAccount()
            ?: false
    }
}
