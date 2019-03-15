package net.mullvad.mullvadvpn

import android.view.View
import android.widget.Button

class ConnectActionButton(val parentView: View) {
    private val button: Button = parentView.findViewById(R.id.action_button)

    var state = ConnectionState.Disconnected
        set(value) {
            when (value) {
                ConnectionState.Disconnected -> disconnected()
                ConnectionState.Connecting -> connecting()
                ConnectionState.Connected -> connected()
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
            ConnectionState.Disconnected -> onConnect?.invoke()
            ConnectionState.Connecting -> onCancel?.invoke()
            ConnectionState.Connected -> onDisconnect?.invoke()
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
