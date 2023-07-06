package net.mullvad.mullvadvpn.ui

import android.content.Context
import android.view.View
import android.widget.TextView
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.model.TunnelState
import net.mullvad.talpid.tunnel.ActionAfterDisconnect

class ConnectionStatus(parentView: View, context: Context) {
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
                    ActionAfterDisconnect.Block -> connected(false)
                    ActionAfterDisconnect.Reconnect -> connecting(false)
                }
            }
            is TunnelState.Disconnected -> disconnected()
            is TunnelState.Connecting -> connecting(state.endpoint?.quantumResistant == true)
            is TunnelState.Connected -> connected(state.endpoint.quantumResistant)
            is TunnelState.Error -> errorState(state.errorState.isBlocking)
        }
    }

    private fun disconnected() {
        spinner.visibility = View.GONE

        text.setTextColor(unsecuredTextColor)
        text.setText(R.string.unsecured_connection)
    }

    private fun connecting(isQuantumResistant: Boolean) {
        spinner.visibility = View.VISIBLE

        text.setTextColor(connectingTextColor)
        text.setText(
            if (isQuantumResistant) {
                R.string.quantum_creating_secure_connection
            } else {
                R.string.creating_secure_connection
            }
        )
    }

    private fun connected(isQuantumResistant: Boolean) {
        spinner.visibility = View.GONE

        text.setTextColor(securedTextColor)
        text.setText(
            if (isQuantumResistant) {
                R.string.quantum_secure_connection
            } else {
                R.string.secure_connection
            }
        )
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
