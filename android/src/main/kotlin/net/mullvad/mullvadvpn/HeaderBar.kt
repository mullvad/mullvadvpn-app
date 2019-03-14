package net.mullvad.mullvadvpn

import android.content.Context
import android.view.View

class HeaderBar(val parentView: View, val context: Context) {
    private val headerBar: View = parentView.findViewById(R.id.header_bar)

    private val securedColor = context.getColor(R.color.green)
    private val unsecuredColor = context.getColor(R.color.red)

    var state = ConnectionState.Disconnected
        set(value) {
            when (value) {
                ConnectionState.Disconnected -> unsecured()
                ConnectionState.Connecting -> secured()
                ConnectionState.Connected -> secured()
            }

            field = value
        }

    private fun unsecured() {
        headerBar.setBackgroundColor(unsecuredColor)
    }

    private fun secured() {
        headerBar.setBackgroundColor(securedColor)
    }
}
