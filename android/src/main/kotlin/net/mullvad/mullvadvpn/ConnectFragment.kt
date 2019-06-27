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

import net.mullvad.mullvadvpn.dataproxy.ConnectionProxy
import net.mullvad.mullvadvpn.dataproxy.LocationInfoCache
import net.mullvad.mullvadvpn.dataproxy.RelayListListener
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
    private lateinit var locationInfoCache: LocationInfoCache
    private lateinit var relayListListener: RelayListListener

    private var updateViewJob: Job? = null

    override fun onAttach(context: Context) {
        super.onAttach(context)

        parentActivity = context as MainActivity
        connectionProxy = parentActivity.connectionProxy
        locationInfoCache = parentActivity.locationInfoCache
        relayListListener = parentActivity.relayListListener
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
        notificationBanner = NotificationBanner(view)
        status = ConnectionStatus(view, context!!)
        locationInfo = LocationInfo(view, locationInfoCache)

        actionButton = ConnectActionButton(view)
        actionButton.apply {
            onConnect = { connectionProxy.connect() }
            onCancel = { connectionProxy.disconnect() }
            onDisconnect = { connectionProxy.disconnect() }
        }

        switchLocationButton = SwitchLocationButton(view)
        switchLocationButton.onClick = { openSwitchLocationScreen() }

        updateView(connectionProxy.uiState)

        connectionProxy.onUiStateChange = { uiState ->
            updateViewJob?.cancel()
            updateViewJob = GlobalScope.launch(Dispatchers.Main) {
                updateView(uiState)
            }
        }

        return view
    }

    override fun onResume() {
        super.onResume()

        relayListListener.onRelayListChange = { relayList, selectedRelayItem ->
            switchLocationButton.location = selectedRelayItem
        }
    }

    override fun onPause() {
        relayListListener.onRelayListChange = null

        super.onPause()
    }

    override fun onDestroyView() {
        locationInfo.onDestroy()
        switchLocationButton.onDestroy()

        connectionProxy.onUiStateChange = null
        updateViewJob?.cancel()

        super.onDestroyView()
    }

    private fun updateView(uiState: TunnelState) {
        val realState = connectionProxy.state

        locationInfoCache.state = realState
        headerBar.setState(realState)

        actionButton.state = uiState
        switchLocationButton.state = uiState

        notificationBanner.setState(uiState)
        status.setState(uiState)
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
