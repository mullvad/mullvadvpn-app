package net.mullvad.mullvadvpn

import android.view.View

class NotificationBanner(val parentView: View) {
    private val banner: View = parentView.findViewById(R.id.notification_banner)

    var state = ConnectionState.Disconnected
        set(value) {
            when (value) {
                ConnectionState.Disconnected -> banner.visibility = View.GONE
                ConnectionState.Connecting -> banner.visibility = View.VISIBLE
                ConnectionState.Connected -> banner.visibility = View.GONE
            }

            field = value
        }
}
