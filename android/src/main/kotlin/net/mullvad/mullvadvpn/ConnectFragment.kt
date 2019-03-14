package net.mullvad.mullvadvpn

import android.os.Bundle
import android.os.Handler
import android.support.v4.app.Fragment
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import android.widget.TextView

class ConnectFragment : Fragment() {
    private lateinit var actionButton: ConnectActionButton
    private lateinit var connectingSpinner: View
    private lateinit var headerBar: HeaderBar
    private lateinit var notificationBanner: View
    private lateinit var status: TextView

    private lateinit var connectHandler: Handler

    private var state = ConnectionState.Disconnected
        set(value) {
            actionButton.state = value
            headerBar.state = value
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

        headerBar = HeaderBar(view, context!!)
        connectingSpinner = view.findViewById(R.id.connecting_spinner)
        notificationBanner = view.findViewById(R.id.notification_banner)
        status = view.findViewById(R.id.connection_status)

        actionButton = ConnectActionButton(view)
        actionButton.onConnect = { connect() }
        actionButton.onCancel = { disconnect() }
        actionButton.onDisconnect = { disconnect() }

        return view
    }

    private fun connect() {
        state = ConnectionState.Connecting

        connectingSpinner.visibility = View.VISIBLE
        notificationBanner.visibility = View.VISIBLE

        status.setTextColor(context!!.getColor(R.color.white))
        status.setText(R.string.creating_secure_connection)

        connectHandler.postDelayed(Runnable { connected() }, 1000)
    }

    private fun disconnect() {
        state = ConnectionState.Disconnected

        connectingSpinner.visibility = View.INVISIBLE
        notificationBanner.visibility = View.GONE

        status.setTextColor(context!!.getColor(R.color.red))
        status.setText(R.string.unsecured_connection)

        connectHandler.removeCallbacksAndMessages(null)
    }

    private fun connected() {
        state = ConnectionState.Connected

        connectingSpinner.visibility = View.INVISIBLE
        notificationBanner.visibility = View.GONE

        status.setTextColor(context!!.getColor(R.color.green))
        status.setText(R.string.secure_connection)
    }
}
