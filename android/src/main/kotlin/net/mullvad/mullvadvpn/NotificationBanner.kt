package net.mullvad.mullvadvpn

import android.view.View

class NotificationBanner(val parentView: View) {
    private val banner: View = parentView.findViewById(R.id.notification_banner)
    private var visible = false

    var state = ConnectionState.Disconnected
        set(value) {
            when (value) {
                ConnectionState.Disconnected -> hide()
                ConnectionState.Connecting -> show()
                ConnectionState.Connected -> hide()
            }

            field = value
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
