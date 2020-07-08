package net.mullvad.mullvadvpn.ui

import android.content.Context
import android.view.View
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.model.TunnelState

class HeaderBar(val parentView: View, context: Context) {
    private val headerBar: View = parentView.findViewById(R.id.header_bar)

    private val securedColor = context.getColor(R.color.green)
    private val unsecuredColor = context.getColor(R.color.red)

    fun setState(state: TunnelState) {
        when (state) {
            is TunnelState.Disconnected -> unsecured()
            is TunnelState.Connecting -> secured()
            is TunnelState.Connected -> secured()
            is TunnelState.Disconnecting -> secured()
            is TunnelState.Error -> {
                if (state.errorState.isBlocking) {
                    secured()
                } else {
                    unsecured()
                }
            }
        }
    }

    private fun unsecured() {
        headerBar.setBackgroundColor(unsecuredColor)
    }

    private fun secured() {
        headerBar.setBackgroundColor(securedColor)
    }
}
