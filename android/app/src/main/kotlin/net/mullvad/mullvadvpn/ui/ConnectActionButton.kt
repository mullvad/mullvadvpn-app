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

    private var showReconnectButton = false
        set(value) {
            if (field != value) {
                field = value
                updateReconnectButton()
            }
        }

    private var reconnectButtonSpace = 0
        set(value) {
            if (field != value) {
                field = value
                updateReconnectButton()
            }
        }

    var tunnelState: TunnelState = TunnelState.Disconnected
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
                is TunnelState.Error -> {
                    if (value.errorState.isBlocking) {
                        connected()
                    } else {
                        blockError()
                    }
                }
            }

            field = value
        }

    var onConnect: (() -> Unit)? = null
    var onCancel: (() -> Unit)? = null
    var onReconnect: (() -> Unit)? = null
    var onDisconnect: (() -> Unit)? = null

    init {
        mainButton.setOnClickListener { action() }
        reconnectButton.setOnClickListener { onReconnect?.invoke() }

        reconnectButton.addOnLayoutChangeListener { _, left, _, right, _, _, _, _, _ ->
            val width = right - left
            val layoutParams = reconnectButton.layoutParams
            val leftMargin =
                when (layoutParams) {
                    is MarginLayoutParams -> layoutParams.leftMargin
                    else -> 0
                }

            reconnectButtonSpace = width + leftMargin
        }
    }

    private fun action() {
        val state = tunnelState

        when (state) {
            is TunnelState.Disconnected -> onConnect?.invoke()
            is TunnelState.Disconnecting -> onConnect?.invoke()
            is TunnelState.Connecting -> onCancel?.invoke()
            is TunnelState.Connected -> onDisconnect?.invoke()
            is TunnelState.Error -> {
                if (state.errorState.isBlocking) {
                    onDisconnect?.invoke()
                } else {
                    onCancel?.invoke()
                }
            }
        }
    }

    private fun disconnected() {
        mainButton.background = greenBackground
        mainButton.setText(R.string.connect)
        showReconnectButton = false
    }

    private fun connecting() {
        redButton(R.string.cancel)
    }

    private fun connected() {
        redButton(R.string.disconnect)
    }

    private fun blockError() {
        redButton(R.string.dismiss)
    }

    private fun redButton(text: Int) {
        mainButton.background = leftRedBackground
        mainButton.setText(text)
        showReconnectButton = true
    }

    private fun updateReconnectButton() {
        if (showReconnectButton) {
            reconnectButton.visibility = View.VISIBLE
            mainButton.setPadding(reconnectButtonSpace, 0, 0, 0)
        } else {
            reconnectButton.visibility = View.GONE
            mainButton.setPadding(0, 0, 0, 0)
        }
    }
}
