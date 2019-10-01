package net.mullvad.mullvadvpn

import android.content.res.Resources
import android.view.View
import net.mullvad.mullvadvpn.model.TunnelState

class HeaderBar(val parentView: View, val resources: Resources) {
    private val headerBar: View = parentView.findViewById(R.id.header_bar)

    private val securedColor = resources.getColor(R.color.green)
    private val unsecuredColor = resources.getColor(R.color.red)

    fun setState(state: TunnelState) {
        when (state) {
            is TunnelState.Disconnected -> unsecured()
            is TunnelState.Connecting -> secured()
            is TunnelState.Connected -> secured()
            is TunnelState.Disconnecting -> secured()
            is TunnelState.Blocked -> secured()
        }
    }

    private fun unsecured() {
        headerBar.setBackgroundColor(unsecuredColor)
    }

    private fun secured() {
        headerBar.setBackgroundColor(securedColor)
    }
}
