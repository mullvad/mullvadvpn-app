package net.mullvad.mullvadvpn

import android.content.Context
import android.view.View

import net.mullvad.mullvadvpn.model.TunnelStateTransition

class HeaderBar(val parentView: View, val context: Context) {
    private val headerBar: View = parentView.findViewById(R.id.header_bar)

    private val securedColor = context.getColor(R.color.green)
    private val unsecuredColor = context.getColor(R.color.red)

    fun setState(state: TunnelStateTransition) {
        when (state) {
            is TunnelStateTransition.Disconnected -> unsecured()
            is TunnelStateTransition.Connecting -> secured()
            is TunnelStateTransition.Connected -> secured()
            is TunnelStateTransition.Disconnecting -> secured()
            is TunnelStateTransition.Blocked -> secured()
        }
    }

    private fun unsecured() {
        headerBar.setBackgroundColor(unsecuredColor)
    }

    private fun secured() {
        headerBar.setBackgroundColor(securedColor)
    }
}
