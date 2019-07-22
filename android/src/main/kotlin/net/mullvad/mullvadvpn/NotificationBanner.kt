package net.mullvad.mullvadvpn

import android.view.View

import net.mullvad.mullvadvpn.model.KeygenEvent
import net.mullvad.mullvadvpn.model.TunnelState

class NotificationBanner(val parentView: View) {
    private val banner: View = parentView.findViewById(R.id.notification_banner)

    private var showingKeyState = false
    private var visible = false

    var keyState: KeygenEvent? = null
        set(value) {
            if (value != field) {
                field = value
                updateBasedOnKeyState()
            }
        }

    var tunnelState: TunnelState = TunnelState.Disconnected()
        set(value) {
            field = value

            if (!showingKeyState) {
                updateBasedOnTunnelState()
            }
        }

    private fun updateBasedOnKeyState() {
        val state = keyState

        if (state == null || state is KeygenEvent.NewKey) {
            // Only update based on tunnel state if it wasn't already showing the tunnel state
            if (showingKeyState) {
                updateBasedOnTunnelState()
            }
        } else {
            showingKeyState = true

            when (state) {
                is KeygenEvent.TooManyKeys -> show()
                is KeygenEvent.GenerationFailure -> show()
            }
        }
    }

    private fun updateBasedOnTunnelState() {
        showingKeyState = false

        when (tunnelState) {
            is TunnelState.Disconnecting -> hide()
            is TunnelState.Disconnected -> hide()
            is TunnelState.Connecting -> show()
            is TunnelState.Connected -> hide()
            is TunnelState.Blocked -> show()
        }
    }

    private fun show() {
        if (!visible) {
            visible = true
            banner.visibility = View.VISIBLE
            banner.translationY = -banner.height.toFloat()
            banner.animate().translationY(0.0F).setDuration(350).start()
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
