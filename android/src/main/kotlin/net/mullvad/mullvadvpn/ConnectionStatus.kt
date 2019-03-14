package net.mullvad.mullvadvpn

import android.content.Context
import android.view.View
import android.widget.TextView

class ConnectionStatus(val parentView: View, val context: Context) {
    private val spinner: View = parentView.findViewById(R.id.connecting_spinner)
    private val text: TextView = parentView.findViewById(R.id.connection_status)

    private val disconnectedTextColor = context.getColor(R.color.red)
    private val connectingTextColor = context.getColor(R.color.white)
    private val connectedTextColor = context.getColor(R.color.green)

    var state = ConnectionState.Disconnected
        set(value) {
            when (value) {
                ConnectionState.Disconnected -> disconnected()
                ConnectionState.Connecting -> connecting()
                ConnectionState.Connected -> connected()
            }

            field = value
        }

    private fun disconnected() {
        spinner.visibility = View.GONE

        text.setTextColor(disconnectedTextColor)
        text.setText(R.string.creating_secure_connection)
    }

    private fun connecting() {
        spinner.visibility = View.VISIBLE

        text.setTextColor(connectingTextColor)
        text.setText(R.string.unsecured_connection)
    }

    private fun connected() {
        spinner.visibility = View.GONE

        text.setTextColor(connectedTextColor)
        text.setText(R.string.secure_connection)
    }
}
