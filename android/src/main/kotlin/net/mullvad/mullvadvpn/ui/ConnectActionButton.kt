package net.mullvad.mullvadvpn.ui

import android.view.View
import android.widget.Button
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.model.TunnelState
import net.mullvad.talpid.tunnel.ActionAfterDisconnect

class ConnectActionButton(val parentView: View) {
    private val mainButton: Button = parentView.findViewById(R.id.action_button)

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
                        ActionAfterDisconnect.Nothing -> disconnected()
                        ActionAfterDisconnect.Block -> connected()
                        ActionAfterDisconnect.Reconnect -> connecting()
                    }
                }
                is TunnelState.Connecting -> connecting()
                is TunnelState.Connected -> connected()
                is TunnelState.Error -> connected()
            }

            field = value
        }

    var onConnect: (() -> Unit)? = null
    var onCancel: (() -> Unit)? = null
    var onDisconnect: (() -> Unit)? = null

    init {
        mainButton.setOnClickListener { action() }
    }

    private fun action() {
        when (tunnelState) {
            is TunnelState.Disconnected -> onConnect?.invoke()
            is TunnelState.Disconnecting -> onConnect?.invoke()
            is TunnelState.Connecting -> onCancel?.invoke()
            is TunnelState.Connected -> onDisconnect?.invoke()
            is TunnelState.Error -> onDisconnect?.invoke()
        }
    }

    private fun disconnected() {
        mainButton.background = greenBackground
        mainButton.setText(R.string.connect)
    }

    private fun connecting() {
        mainButton.background = transparentRedBackground
        mainButton.setText(R.string.cancel)
    }

    private fun connected() {
        mainButton.background = transparentRedBackground
        mainButton.setText(R.string.disconnect)
    }
}
