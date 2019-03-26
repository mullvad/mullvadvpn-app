package net.mullvad.mullvadvpn

import android.view.View

import net.mullvad.mullvadvpn.model.TunnelStateTransition

class NotificationBanner(val parentView: View) {
    private val banner: View = parentView.findViewById(R.id.notification_banner)
    private var visible = false

    fun setState(state: TunnelStateTransition) {
        when (state) {
            is TunnelStateTransition.Disconnecting -> hide()
            is TunnelStateTransition.Disconnected -> hide()
            is TunnelStateTransition.Connecting -> show()
            is TunnelStateTransition.Connected -> hide()
            is TunnelStateTransition.Blocked -> show()
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
