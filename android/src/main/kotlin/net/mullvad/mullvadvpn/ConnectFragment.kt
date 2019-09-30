package net.mullvad.mullvadvpn

import android.content.Context
import android.os.Bundle
import android.support.v4.app.Fragment
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import android.widget.ImageButton
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.dataproxy.AppVersionInfoCache
import net.mullvad.mullvadvpn.dataproxy.ConnectionProxy
import net.mullvad.mullvadvpn.dataproxy.KeyStatusListener
import net.mullvad.mullvadvpn.dataproxy.LocationInfoCache
import net.mullvad.mullvadvpn.dataproxy.RelayListListener
import net.mullvad.mullvadvpn.model.KeygenEvent
import net.mullvad.mullvadvpn.model.TunnelState
import net.mullvad.mullvadvpn.util.SmartDeferred

val KEY_IS_TUNNEL_INFO_EXPANDED = "is_tunnel_info_expanded"

class ConnectFragment : Fragment() {
    private lateinit var actionButton: ConnectActionButton
    private lateinit var switchLocationButton: SwitchLocationButton
    private lateinit var headerBar: HeaderBar
    private lateinit var notificationBanner: NotificationBanner
    private lateinit var status: ConnectionStatus
    private lateinit var locationInfo: LocationInfo

    private lateinit var parentActivity: MainActivity
    private lateinit var connectionProxy: SmartDeferred<ConnectionProxy>
    private lateinit var keyStatusListener: KeyStatusListener
    private lateinit var locationInfoCache: LocationInfoCache
    private lateinit var relayListListener: RelayListListener
    private lateinit var versionInfoCache: AppVersionInfoCache

    private lateinit var updateKeyStatusJob: Job
    private var updateTunnelStateJob: Job? = null
    private var tunnelStateSubscriptionJob: Long? = null

    private var isTunnelInfoExpanded = false
    private var tunnelStateListener: Int? = null

    override fun onAttach(context: Context) {
        super.onAttach(context)

        parentActivity = context as MainActivity
        connectionProxy = parentActivity.connectionProxy
        keyStatusListener = parentActivity.keyStatusListener
        locationInfoCache = parentActivity.locationInfoCache
        relayListListener = parentActivity.relayListListener
        versionInfoCache = parentActivity.appVersionInfoCache
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        isTunnelInfoExpanded =
            savedInstanceState?.getBoolean(KEY_IS_TUNNEL_INFO_EXPANDED, false) ?: false
    }

    override fun onCreateView(
        inflater: LayoutInflater,
        container: ViewGroup?,
        savedInstanceState: Bundle?
    ): View {
        val view = inflater.inflate(R.layout.connect, container, false)

        view.findViewById<ImageButton>(R.id.settings).setOnClickListener {
            parentActivity.openSettings()
        }

        headerBar = HeaderBar(view, context!!)
        notificationBanner = NotificationBanner(view, context!!, versionInfoCache)
        status = ConnectionStatus(view, context!!)

        locationInfo = LocationInfo(view, context!!)
        locationInfo.isTunnelInfoExpanded = isTunnelInfoExpanded

        actionButton = ConnectActionButton(view)
        actionButton.apply {
            onConnect = { connectionProxy.awaitThen { connect() } }
            onCancel = { connectionProxy.awaitThen { disconnect() } }
            onDisconnect = { connectionProxy.awaitThen { disconnect() } }
        }

        switchLocationButton = SwitchLocationButton(view, resources)
        switchLocationButton.onClick = { openSwitchLocationScreen() }

        updateKeyStatusJob = updateKeyStatus(keyStatusListener.keyStatus)

        return view
    }

    override fun onResume() {
        super.onResume()

        locationInfo.isTunnelInfoExpanded = isTunnelInfoExpanded

        notificationBanner.onResume()

        keyStatusListener.onKeyStatusChange = { keyStatus ->
            updateKeyStatusJob.cancel()
            updateKeyStatusJob = updateKeyStatus(keyStatus)
        }

        locationInfoCache.onNewLocation = { location ->
            locationInfo.location = location
        }

        relayListListener.onRelayListChange = { relayList, selectedRelayItem ->
            switchLocationButton.location = selectedRelayItem
        }

        tunnelStateSubscriptionJob = connectionProxy.awaitThen {
            tunnelStateListener = onUiStateChange.subscribe { uiState ->
                updateTunnelStateJob?.cancel()
                updateTunnelStateJob = updateTunnelState(uiState, state)
            }
        }
    }

    override fun onPause() {
        keyStatusListener.onKeyStatusChange = null
        locationInfoCache.onNewLocation = null
        relayListListener.onRelayListChange = null

        tunnelStateSubscriptionJob?.let { jobId ->
            connectionProxy.cancelJob(jobId)
        }

        tunnelStateListener?.let { listener ->
            connectionProxy.awaitThen {
                onUiStateChange.unsubscribe(listener)
            }
        }

        updateTunnelStateJob?.cancel()
        notificationBanner.onPause()

        isTunnelInfoExpanded = locationInfo.isTunnelInfoExpanded

        super.onPause()
    }

    override fun onDestroyView() {
        switchLocationButton.onDestroy()

        super.onDestroyView()
    }

    override fun onSaveInstanceState(state: Bundle) {
        isTunnelInfoExpanded = locationInfo.isTunnelInfoExpanded
        state.putBoolean(KEY_IS_TUNNEL_INFO_EXPANDED, isTunnelInfoExpanded)
    }

    private fun updateTunnelState(uiState: TunnelState, realState: TunnelState) =
        GlobalScope.launch(Dispatchers.Main) {
        notificationBanner.tunnelState = realState
        locationInfoCache.state = realState
        locationInfo.state = realState
        headerBar.setState(realState)
        status.setState(realState)

        actionButton.tunnelState = uiState
        switchLocationButton.state = uiState
    }

    private fun updateKeyStatus(keyStatus: KeygenEvent?) = GlobalScope.launch(Dispatchers.Main) {
        notificationBanner.keyState = keyStatus
    }

    private fun openSwitchLocationScreen() {
        fragmentManager?.beginTransaction()?.apply {
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
}
