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

    private var canConnect = true
        set(value) {
            field = value
            updateEnabled()
        }

    private var showingConnect = true
        set(value) {
            field = value
            updateEnabled()
        }

    var keyState: KeygenEvent? = null
        set(value) {
            when (value) {
                null -> canConnect = true
                is KeygenEvent.NewKey -> canConnect = true
                is KeygenEvent.TooManyKeys -> canConnect = false
                is KeygenEvent.GenerationFailure -> canConnect = false
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
        showingConnect = true
    }

    private fun connecting() {
        button.setBackgroundResource(R.drawable.transparent_red_button_background)
        button.setText(R.string.cancel)
        showingConnect = false
    }

    private fun connected() {
        button.setBackgroundResource(R.drawable.transparent_red_button_background)
        button.setText(R.string.disconnect)
        showingConnect = false
    }

    private fun updateEnabled() {
        enabled = !showingConnect || canConnect
    }
}
