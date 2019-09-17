package net.mullvad.mullvadvpn

import android.view.View
import android.widget.Button

import net.mullvad.mullvadvpn.model.ActionAfterDisconnect
import net.mullvad.mullvadvpn.model.KeygenEvent
import net.mullvad.mullvadvpn.model.TunnelState

class ConnectActionButton(val parentView: View) {
    private val button: Button = parentView.findViewById(R.id.action_button)

    private val resources = parentView.context.resources
    private val greenBackground = resources.getDrawable(R.drawable.green_button_background, null)
    private val transparentRedBackground =
        resources.getDrawable(R.drawable.transparent_red_button_background, null)

    var tunnelState: TunnelState = TunnelState.Disconnected()
        set(value) {
            when (value) {
                is TunnelState.Disconnected -> disconnected()
                is TunnelState.Disconnecting -> {
                    when (value.actionAfterDisconnect) {
                        is ActionAfterDisconnect.Nothing -> disconnected()
                        is ActionAfterDisconnect.Block -> connected()
                        is ActionAfterDisconnect.Reconnect -> connecting()
                    }
                }
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
        button.background = greenBackground
        button.setText(R.string.connect)
    }

    private fun connecting() {
        button.background = transparentRedBackground
        button.setText(R.string.cancel)
    }

    private fun connected() {
        button.background = transparentRedBackground
        button.setText(R.string.disconnect)
    }
}
