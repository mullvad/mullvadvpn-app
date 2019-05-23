package net.mullvad.mullvadvpn

import android.view.View
import android.widget.Button

import net.mullvad.mullvadvpn.model.TunnelStateTransition

class ConnectActionButton(val parentView: View) {
    private val button: Button = parentView.findViewById(R.id.action_button)

    var state: TunnelStateTransition = TunnelStateTransition.Disconnected()
        set(value) {
            when (value) {
                is TunnelStateTransition.Disconnected -> disconnected()
                is TunnelStateTransition.Disconnecting -> disconnected()
                is TunnelStateTransition.Connecting -> connecting()
                is TunnelStateTransition.Connected -> connected()
                is TunnelStateTransition.Blocked -> connected()
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
            is TunnelStateTransition.Disconnected -> onConnect?.invoke()
            is TunnelStateTransition.Disconnecting -> onConnect?.invoke()
            is TunnelStateTransition.Connecting -> onCancel?.invoke()
            is TunnelStateTransition.Connected -> onDisconnect?.invoke()
            is TunnelStateTransition.Blocked -> onDisconnect?.invoke()
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
