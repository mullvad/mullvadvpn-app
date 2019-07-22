package net.mullvad.mullvadvpn

import android.view.View
import android.widget.Button

import net.mullvad.mullvadvpn.model.KeygenEvent
import net.mullvad.mullvadvpn.model.TunnelState

class ConnectActionButton(val parentView: View) {
    private val button: Button = parentView.findViewById(R.id.action_button)

    private var enabled = true
        set(value) {
            if (field != value) {
                field = value

                button.setEnabled(value)
                button.setAlpha(if (value) 1.0F else 0.5F)
            }
        }

    var keyState: KeygenEvent? = null
        set(value) {
            when (value) {
                null -> enabled = true
                is KeygenEvent.NewKey -> enabled = true
                is KeygenEvent.TooManyKeys -> enabled = false
                is KeygenEvent.GenerationFailure -> enabled = false
            }

            field = value
        }

    var tunnelState: TunnelState = TunnelState.Disconnected()
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
        when (tunnelState) {
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
