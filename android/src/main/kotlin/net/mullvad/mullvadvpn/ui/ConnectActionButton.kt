package net.mullvad.mullvadvpn.ui

import android.view.View
import android.view.ViewGroup.MarginLayoutParams
import android.widget.Button
import android.widget.ImageButton
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.model.TunnelState
import net.mullvad.talpid.tunnel.ActionAfterDisconnect

class ConnectActionButton(val parentView: View) {
    private val mainButton: Button = parentView.findViewById(R.id.action_button)
    private val reconnectButton: ImageButton = parentView.findViewById(R.id.reconnect_button)

    private val resources = parentView.context.resources
    private val greenBackground = resources.getDrawable(R.drawable.green_button_background, null)
    private val leftRedBackground =
        resources.getDrawable(R.drawable.transparent_red_left_half_button_background, null)

    private var reconnectButtonSpace = 0

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

        reconnectButton.addOnLayoutChangeListener { _, left, _, right, _, _, _, _, _ ->
            val width = right - left
            val layoutParams = reconnectButton.layoutParams
            val leftMargin = when (layoutParams) {
                is MarginLayoutParams -> layoutParams.leftMargin
                else -> 0
            }

            reconnectButtonSpace = width + leftMargin
        }
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
        reconnectButton.visibility = View.GONE
        mainButton.setPadding(0, 0, 0, 0)
        mainButton.background = greenBackground
        mainButton.setText(R.string.connect)
    }

    private fun connecting() {
        reconnectButton.visibility = View.VISIBLE
        mainButton.setPadding(reconnectButtonSpace, 0, 0, 0)
        mainButton.background = leftRedBackground
        mainButton.setText(R.string.cancel)
    }

    private fun connected() {
        reconnectButton.visibility = View.VISIBLE
        mainButton.setPadding(reconnectButtonSpace, 0, 0, 0)
        mainButton.background = leftRedBackground
        mainButton.setText(R.string.disconnect)
    }
}
