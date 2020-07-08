package net.mullvad.mullvadvpn.ui

import android.content.Context
import android.view.View
import android.widget.TextView
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.model.TunnelState
import net.mullvad.talpid.tunnel.ActionAfterDisconnect

class ConnectionStatus(val parentView: View, context: Context) {
    private val spinner: View = parentView.findViewById(R.id.connecting_spinner)
    private val text: TextView = parentView.findViewById(R.id.connection_status)

    private val unsecuredTextColor = context.getColor(R.color.red)
    private val connectingTextColor = context.getColor(R.color.white)
    private val securedTextColor = context.getColor(R.color.green)

    fun setState(state: TunnelState) {
        when (state) {
            is TunnelState.Disconnecting -> {
                when (state.actionAfterDisconnect) {
                    ActionAfterDisconnect.Nothing -> disconnected()
                    ActionAfterDisconnect.Block -> connected()
                    ActionAfterDisconnect.Reconnect -> connecting()
                }
            }
            is TunnelState.Disconnected -> disconnected()
            is TunnelState.Connecting -> connecting()
            is TunnelState.Connected -> connected()
            is TunnelState.Error -> errorState(state.errorState.isBlocking)
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

    private fun errorState(isBlocking: Boolean) {
        spinner.visibility = View.GONE

        if (isBlocking) {
            text.setTextColor(securedTextColor)
            text.setText(R.string.blocked_connection)
        } else {
            text.setTextColor(unsecuredTextColor)
            text.setText(R.string.error_state)
        }
    }
}
