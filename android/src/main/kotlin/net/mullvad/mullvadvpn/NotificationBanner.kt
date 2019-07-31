package net.mullvad.mullvadvpn

import android.widget.TextView
import android.view.View

import net.mullvad.mullvadvpn.model.ActionAfterDisconnect
import net.mullvad.mullvadvpn.model.KeygenEvent
import net.mullvad.mullvadvpn.model.TunnelState

class NotificationBanner(val parentView: View) {
    private val banner: View = parentView.findViewById(R.id.notification_banner)
    private val title: TextView = parentView.findViewById(R.id.notification_title)
    private val message: TextView = parentView.findViewById(R.id.notification_message)

    private var visible = false

    var keyState: KeygenEvent? = null
        set(value) {
            field = value
            update()
        }

    var tunnelState: TunnelState = TunnelState.Disconnected()
        set(value) {
            field = value
            update()
        }

    private fun update() {
        updateBasedOnKeyState() || updateBasedOnTunnelState()
    }

    private fun updateBasedOnKeyState(): Boolean {
        when (keyState) {
            null -> return false
            is KeygenEvent.NewKey -> return false
            is KeygenEvent.TooManyKeys -> show(R.string.too_many_keys, null)
            is KeygenEvent.GenerationFailure -> show(R.string.failed_to_generate_key, null)
        }

        return true
    }

    private fun updateBasedOnTunnelState(): Boolean {
        val state = tunnelState

        when (state) {
            is TunnelState.Disconnecting -> {
                when (state.actionAfterDisconnect) {
                    is ActionAfterDisconnect.Nothing -> hide()
                    is ActionAfterDisconnect.Block -> show(R.string.blocking_internet, null)
                    is ActionAfterDisconnect.Reconnect -> show(R.string.blocking_internet, null)
                }
            }
            is TunnelState.Disconnected -> hide()
            is TunnelState.Connecting -> show(R.string.blocking_internet, null)
            is TunnelState.Connected -> hide()
            is TunnelState.Blocked -> show(R.string.blocking_internet, null)
        }

        return true
    }

    private fun show(titleText: Int, messageText: Int?) {
        if (!visible) {
            visible = true
            banner.visibility = View.VISIBLE
            banner.translationY = -banner.height.toFloat()
            banner.animate().translationY(0.0F).setDuration(350).start()
        }

        title.setText(titleText)

        if (messageText == null) {
            message.visibility = View.GONE
        } else {
            message.setText(messageText)
            message.visibility = View.VISIBLE
        }
    }

    private fun hide() {
        if (visible) {
            visible = false
            banner.animate().translationY(-banner.height.toFloat()).setDuration(350).withEndAction {
                banner.visibility = View.INVISIBLE
            }
        }
    }
}
