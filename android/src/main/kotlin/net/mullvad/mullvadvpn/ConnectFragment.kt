package net.mullvad.mullvadvpn

import android.os.Bundle
import android.os.Handler
import android.support.v4.app.Fragment
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup

class ConnectFragment : Fragment() {
    private lateinit var actionButton: ConnectActionButton
    private lateinit var headerBar: HeaderBar
    private lateinit var notificationBanner: View
    private lateinit var status: ConnectionStatus

    private lateinit var connectHandler: Handler

    private var state = ConnectionState.Disconnected
        set(value) {
            actionButton.state = value
            headerBar.state = value
            status.state = value
            field = value
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

        notificationBanner = view.findViewById(R.id.notification_banner)

        headerBar = HeaderBar(view, context!!)
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

        notificationBanner.visibility = View.VISIBLE

        connectHandler.postDelayed(Runnable { connected() }, 1000)
    }

    private fun disconnect() {
        state = ConnectionState.Disconnected

        notificationBanner.visibility = View.GONE

        connectHandler.removeCallbacksAndMessages(null)
    }

    private fun connected() {
        state = ConnectionState.Connected

        notificationBanner.visibility = View.GONE
    }
}
