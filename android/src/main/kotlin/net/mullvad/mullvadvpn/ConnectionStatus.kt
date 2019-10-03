package net.mullvad.mullvadvpn

import android.content.res.Resources
import android.view.View
import android.widget.TextView
import net.mullvad.mullvadvpn.model.ActionAfterDisconnect
import net.mullvad.mullvadvpn.model.TunnelState

class ConnectionStatus(val parentView: View, val resources: Resources) {
    private val spinner: View = parentView.findViewById(R.id.connecting_spinner)
    private val text: TextView = parentView.findViewById(R.id.connection_status)

    private val unsecuredTextColor = resources.getColor(R.color.red)
    private val connectingTextColor = resources.getColor(R.color.white)
    private val securedTextColor = resources.getColor(R.color.green)

    fun setState(state: TunnelState) {
        when (state) {
            is TunnelState.Disconnecting -> {
                when (state.actionAfterDisconnect) {
                    is ActionAfterDisconnect.Nothing -> disconnected()
                    is ActionAfterDisconnect.Block -> connected()
                    is ActionAfterDisconnect.Reconnect -> connecting()
                }
            }
            is TunnelState.Disconnected -> disconnected()
            is TunnelState.Connecting -> connecting()
            is TunnelState.Connected -> connected()
            is TunnelState.Blocked -> blocked()
        }
    }

    private fun disconnected() {
        spinner.visibility = View.GONE

        text.setTextColor(unsecuredTextColor)
        text.setText(R.string.unsecured_connection)
    }

    private fun connecting() {
        spinner.visibility = View.VISIBLE

        text.setTextColor(connectingTextColor)
        text.setText(R.string.creating_secure_connection)
    }

    private fun connected() {
        spinner.visibility = View.GONE

        text.setTextColor(securedTextColor)
        text.setText(R.string.secure_connection)
    }

    private fun blocked() {
        spinner.visibility = View.GONE

        text.setTextColor(securedTextColor)
        text.setText(R.string.blocked_connection)
    }
}
