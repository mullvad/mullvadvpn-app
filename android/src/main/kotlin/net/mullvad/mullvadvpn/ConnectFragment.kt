package net.mullvad.mullvadvpn

import kotlinx.coroutines.launch
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.Job

import android.content.Context
import android.os.Bundle
import android.support.v4.app.Fragment
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import android.widget.ImageButton

import net.mullvad.mullvadvpn.dataproxy.AppVersionInfoCache
import net.mullvad.mullvadvpn.dataproxy.ConnectionProxy
import net.mullvad.mullvadvpn.dataproxy.KeyStatusListener
import net.mullvad.mullvadvpn.dataproxy.LocationInfoCache
import net.mullvad.mullvadvpn.dataproxy.RelayListListener
import net.mullvad.mullvadvpn.model.KeygenEvent
import net.mullvad.mullvadvpn.model.TunnelState

class ConnectFragment : Fragment() {
    private lateinit var actionButton: ConnectActionButton
    private lateinit var switchLocationButton: SwitchLocationButton
    private lateinit var headerBar: HeaderBar
    private lateinit var notificationBanner: NotificationBanner
    private lateinit var status: ConnectionStatus
    private lateinit var locationInfo: LocationInfo

    private lateinit var parentActivity: MainActivity
    private lateinit var connectionProxy: ConnectionProxy
    private lateinit var keyStatusListener: KeyStatusListener
    private lateinit var locationInfoCache: LocationInfoCache
    private lateinit var relayListListener: RelayListListener
    private lateinit var versionInfoCache: AppVersionInfoCache

    private lateinit var updateKeyStatusJob: Job
    private lateinit var updateTunnelStateJob: Job

    override fun onAttach(context: Context) {
        super.onAttach(context)

        parentActivity = context as MainActivity
        connectionProxy = parentActivity.connectionProxy
        keyStatusListener = parentActivity.keyStatusListener
        locationInfoCache = parentActivity.locationInfoCache
        relayListListener = parentActivity.relayListListener
        versionInfoCache = parentActivity.appVersionInfoCache
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

        actionButton = ConnectActionButton(view)
        actionButton.apply {
            onConnect = { connectionProxy.connect() }
            onCancel = { connectionProxy.disconnect() }
            onDisconnect = { connectionProxy.disconnect() }
        }

        switchLocationButton = SwitchLocationButton(view)
        switchLocationButton.onClick = { openSwitchLocationScreen() }

        updateKeyStatusJob = updateKeyStatus(keyStatusListener.keyStatus)
        updateTunnelStateJob = updateTunnelState(connectionProxy.uiState)

        connectionProxy.onUiStateChange = { uiState ->
            updateTunnelStateJob.cancel()
            updateTunnelStateJob = updateTunnelState(uiState)
        }

        return view
    }

    override fun onResume() {
        super.onResume()

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
    }

    override fun onPause() {
        keyStatusListener.onKeyStatusChange = null
        locationInfoCache.onNewLocation = null
        relayListListener.onRelayListChange = null

        super.onPause()
    }

    override fun onDestroyView() {
        switchLocationButton.onDestroy()

        connectionProxy.onUiStateChange = null
        updateTunnelStateJob.cancel()

        super.onDestroyView()
    }

    private fun updateTunnelState(uiState: TunnelState) = GlobalScope.launch(Dispatchers.Main) {
        val realState = connectionProxy.state

        locationInfoCache.state = realState
        locationInfo.state = realState
        headerBar.setState(realState)

        actionButton.tunnelState = uiState
        switchLocationButton.state = uiState
        notificationBanner.tunnelState = uiState
        status.setState(uiState)
    }

    private fun updateKeyStatus(keyStatus: KeygenEvent?) = GlobalScope.launch(Dispatchers.Main) {
        notificationBanner.keyState = keyStatus
        actionButton.keyState = keyStatus
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
