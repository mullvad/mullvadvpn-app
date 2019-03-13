package net.mullvad.mullvadvpn

import android.os.Bundle
import android.support.v4.app.Fragment
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import android.widget.TextView

class ConnectFragment : Fragment() {
    private lateinit var actionButton: ConnectActionButton
    private lateinit var connectingSpinner: View
    private lateinit var headerBar: View
    private lateinit var notificationBanner: View
    private lateinit var status: TextView

    override fun onCreateView(
        inflater: LayoutInflater,
        container: ViewGroup?,
        savedInstanceState: Bundle?
    ): View {
        val view = inflater.inflate(R.layout.connect, container, false)

        connectingSpinner = view.findViewById(R.id.connecting_spinner)
        headerBar = view.findViewById(R.id.header_bar)
        notificationBanner = view.findViewById(R.id.notification_banner)
        status = view.findViewById(R.id.connection_status)

        actionButton = ConnectActionButton(view)
        actionButton.onConnect = { connect() }

        return view
    }

    private fun connect() {
        actionButton.state = ConnectionState.Connecting

        connectingSpinner.visibility = View.VISIBLE
        notificationBanner.visibility = View.VISIBLE

        headerBar.setBackgroundColor(context!!.getColor(R.color.green))

        status.setTextColor(context!!.getColor(R.color.white))
        status.setText(R.string.creating_secure_connection)
    }
}
