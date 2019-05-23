package net.mullvad.mullvadvpn

import kotlinx.coroutines.launch
import kotlinx.coroutines.Deferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope

import android.content.Context
import android.os.Bundle
import android.os.Handler
import android.support.v4.app.Fragment
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import android.widget.Button

class ConnectFragment : Fragment() {
    private lateinit var actionButton: ConnectActionButton
    private lateinit var headerBar: HeaderBar
    private lateinit var notificationBanner: NotificationBanner
    private lateinit var status: ConnectionStatus

    private lateinit var connectHandler: Handler
    private lateinit var daemon: Deferred<MullvadDaemon>

    private var state = ConnectionState.Disconnected
        set(value) {
            actionButton.state = value
            headerBar.state = value
            notificationBanner.state = value
            status.state = value

            field = value
        }

    override fun onAttach(context: Context) {
        super.onAttach(context)

        daemon = (context as MainActivity).asyncDaemon
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        connectHandler = Handler()
    }

    override fun onCreateView(
        inflater: LayoutInflater,
        container: ViewGroup?,
        savedInstanceState: Bundle?
    ): View {
        val view = inflater.inflate(R.layout.connect, container, false)

        view.findViewById<Button>(R.id.switch_location).setOnClickListener {
            openSwitchLocationScreen()
        }

        headerBar = HeaderBar(view, context!!)
        notificationBanner = NotificationBanner(view)
        status = ConnectionStatus(view, context!!)

        actionButton = ConnectActionButton(view)
        actionButton.apply {
            onConnect = { connect() }
            onCancel = { disconnect() }
            onDisconnect = { disconnect() }
        }

        return view
    }

    private fun connect() {
        state = ConnectionState.Connecting

        GlobalScope.launch(Dispatchers.Default) {
            daemon.await().connect()
        }

        connectHandler.postDelayed(Runnable { connected() }, 1000)
    }

    private fun disconnect() {
        state = ConnectionState.Disconnected

        GlobalScope.launch(Dispatchers.Default) {
            daemon.await().disconnect()
        }

        connectHandler.removeCallbacksAndMessages(null)
    }

    private fun connected() {
        state = ConnectionState.Connected
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
