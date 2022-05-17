package net.mullvad.mullvadvpn.ui

import android.os.Bundle
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import androidx.core.content.ContextCompat
import androidx.lifecycle.lifecycleScope
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.collect
import kotlinx.coroutines.flow.map
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.model.TunnelState
import net.mullvad.mullvadvpn.ui.notification.AccountExpiryNotification
import net.mullvad.mullvadvpn.ui.notification.KeyStatusNotification
import net.mullvad.mullvadvpn.ui.notification.TunnelStateNotification
import net.mullvad.mullvadvpn.ui.notification.VersionInfoNotification
import net.mullvad.mullvadvpn.ui.widget.HeaderBar
import net.mullvad.mullvadvpn.ui.widget.NotificationBanner
import net.mullvad.mullvadvpn.ui.widget.SwitchLocationButton
import org.joda.time.DateTime

val KEY_IS_TUNNEL_INFO_EXPANDED = "is_tunnel_info_expanded"

class ConnectFragment :
    ServiceDependentFragment(OnNoService.GoToLaunchScreen), NavigationBarPainter {
    private lateinit var actionButton: ConnectActionButton
    private lateinit var switchLocationButton: SwitchLocationButton
    private lateinit var headerBar: HeaderBar
    private lateinit var notificationBanner: NotificationBanner
    private lateinit var status: ConnectionStatus
    private lateinit var locationInfo: LocationInfo

    private var isTunnelInfoExpanded = false

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        isTunnelInfoExpanded =
            savedInstanceState?.getBoolean(KEY_IS_TUNNEL_INFO_EXPANDED, false) ?: false
    }

    override fun onSafelyCreateView(
        inflater: LayoutInflater,
        container: ViewGroup?,
        savedInstanceState: Bundle?
    ): View {
        val view = inflater.inflate(R.layout.connect, container, false)

        headerBar = view.findViewById<HeaderBar>(R.id.header_bar).apply {
            tunnelState = TunnelState.Disconnected
        }

        notificationBanner = view.findViewById<NotificationBanner>(R.id.notification_banner).apply {
            notifications.apply {
                register(TunnelStateNotification(parentActivity, connectionProxy))
                register(KeyStatusNotification(parentActivity, authTokenCache, keyStatusListener))
                register(VersionInfoNotification(parentActivity, appVersionInfoCache))
                register(AccountExpiryNotification(parentActivity, authTokenCache, accountCache))
            }
        }

        status = ConnectionStatus(view, parentActivity)

        locationInfo = LocationInfo(view, requireContext())
        locationInfo.isTunnelInfoExpanded = isTunnelInfoExpanded

        actionButton = ConnectActionButton(view)
        actionButton.apply {
            onConnect = { connectionProxy.connect() }
            onCancel = { connectionProxy.disconnect() }
            onReconnect = { connectionProxy.reconnect() }
            onDisconnect = { connectionProxy.disconnect() }
        }

        switchLocationButton = view.findViewById<SwitchLocationButton>(R.id.switch_location).apply {
            onClick = { openSwitchLocationScreen() }
        }

        return view
    }

    override fun onSafelyStart() {
        locationInfo.isTunnelInfoExpanded = isTunnelInfoExpanded

        notificationBanner.onResume()

        locationInfoCache.onNewLocation = { location ->
            jobTracker.newUiJob("updateLocationInfo") {
                locationInfo.location = location
            }
        }

        relayListListener.onRelayListChange = { _, selectedRelayItem ->
            jobTracker.newUiJob("updateSelectedRelayItem") {
                switchLocationButton.location = selectedRelayItem
            }
        }

        connectionProxy.onUiStateChange.subscribe(this) { uiState ->
            viewLifecycleOwner.lifecycleScope.launchWhenStarted {
                updateTunnelState(uiState, connectionProxy.state)
            }
        }

        jobTracker.newUiJob("updateAccountExpiry") {
            accountCache.accountExpiryState
                .map { state -> state.date() }
                .collect { expiryDate ->
                    if (expiryDate?.isBeforeNow == true) {
                        openOutOfTimeScreen()
                    } else if (expiryDate != null)
                        scheduleNextAccountExpiryCheck(expiryDate)
                }
        }
    }

    override fun onSafelyStop() {
        jobTracker.cancelJob("updateAccountExpiry")

        locationInfoCache.onNewLocation = null
        relayListListener.onRelayListChange = null

        keyStatusListener.onKeyStatusChange.unsubscribe(this)
        connectionProxy.onUiStateChange.unsubscribe(this)

        notificationBanner.onPause()

        isTunnelInfoExpanded = locationInfo.isTunnelInfoExpanded
    }

    override fun onSafelyDestroyView() {
        notificationBanner.onDestroy()
    }

    override fun onSafelySaveInstanceState(state: Bundle) {
        isTunnelInfoExpanded = locationInfo.isTunnelInfoExpanded
        state.putBoolean(KEY_IS_TUNNEL_INFO_EXPANDED, isTunnelInfoExpanded)
    }

    override fun onResume() {
        super.onResume()
        paintNavigationBar(ContextCompat.getColor(requireContext(), R.color.blue))
    }

    private fun updateTunnelState(uiState: TunnelState, realState: TunnelState) {
        locationInfo.state = realState
        headerBar.tunnelState = realState
        status.setState(realState)

        actionButton.tunnelState = uiState
        switchLocationButton.tunnelState = uiState
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
            commit()
        }
    }

    private fun openOutOfTimeScreen() {
        jobTracker.newUiJob("openOutOfTimeScreen") {
            parentFragmentManager.beginTransaction().apply {
                replace(R.id.main_fragment, OutOfTimeFragment())
                commit()
            }
        }
    }

    private fun scheduleNextAccountExpiryCheck(expiration: DateTime) {
        jobTracker.newBackgroundJob("refetchAccountExpiry") {
            val millisUntilExpiration = expiration.millis - DateTime.now().millis

            delay(millisUntilExpiration)
            accountCache.fetchAccountExpiry()

            // If the account ran out of time but is still connected, fetching the expiry again will
            // fail. Therefore, after a timeout of 5 seconds the app will assume the account time
            // really expired and move to the out of time screen. However, if fetching the expiry
            // succeeds, this job is cancelled and replaced with a new scheduled check.
            delay(5_000)
            openOutOfTimeScreen()
        }
    }
}
