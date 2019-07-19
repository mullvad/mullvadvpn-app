package net.mullvad.mullvadvpn

import android.view.View

import net.mullvad.mullvadvpn.model.TunnelState

class NotificationBanner(val parentView: View) {
    private val banner: View = parentView.findViewById(R.id.notification_banner)
    private var visible = false

    var tunnelState: TunnelState = TunnelState.Disconnected()
        set(value) {
            field = value
            update()
        }

    private fun update() {
        synchronized(this) {
            when (tunnelState) {
                is TunnelState.Disconnecting -> hide()
                is TunnelState.Disconnected -> hide()
                is TunnelState.Connecting -> show()
                is TunnelState.Connected -> hide()
                is TunnelState.Blocked -> show()
            }
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
