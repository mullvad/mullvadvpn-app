package net.mullvad.mullvadvpn

import android.view.View
import android.widget.Button

import net.mullvad.mullvadvpn.model.TunnelState

class ConnectActionButton(val parentView: View) {
    private val button: Button = parentView.findViewById(R.id.action_button)

    var state: TunnelState = TunnelState.Disconnected()
        set(value) {
            when (value) {
                is TunnelState.Disconnected -> disconnected()
                is TunnelState.Disconnecting -> disconnected()
                is TunnelState.Connecting -> connecting()
                is TunnelState.Connected -> connected()
                is TunnelState.Blocked -> connected()
            }

            field = value
        }

    var onConnect: (() -> Unit)? = null
    var onCancel: (() -> Unit)? = null
    var onDisconnect: (() -> Unit)? = null

    init {
        button.setOnClickListener { action() }
    }

    private fun action() {
        when (state) {
            is TunnelState.Disconnected -> onConnect?.invoke()
            is TunnelState.Disconnecting -> onConnect?.invoke()
            is TunnelState.Connecting -> onCancel?.invoke()
            is TunnelState.Connected -> onDisconnect?.invoke()
            is TunnelState.Blocked -> onDisconnect?.invoke()
        }
    }

    private fun disconnected() {
        button.setBackgroundResource(R.drawable.green_button_background)
        button.setText(R.string.connect)
    }

    private fun connecting() {
        button.setBackgroundResource(R.drawable.transparent_red_button_background)
        button.setText(R.string.cancel)
    }

    private fun connected() {
        button.setBackgroundResource(R.drawable.transparent_red_button_background)
        button.setText(R.string.disconnect)
    }
}
