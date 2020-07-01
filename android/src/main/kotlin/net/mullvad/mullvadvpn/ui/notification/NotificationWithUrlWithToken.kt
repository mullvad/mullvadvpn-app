package net.mullvad.mullvadvpn.ui.notification

import android.content.Context
import android.content.Intent
import android.net.Uri
import net.mullvad.mullvadvpn.service.MullvadDaemon

abstract class NotificationWithUrlWithToken(
    protected val context: Context,
    protected val daemon: MullvadDaemon,
    urlId: Int
) : InAppNotification() {
    private val url = context.getString(urlId)

    protected val openUrl: suspend () -> Unit = {
        context.startActivity(Intent(Intent.ACTION_VIEW, buildUrl()))
    }

    init {
        onClick = openUrl
        showIcon = true
    }

    private fun buildUrl() = Uri.parse("$url?token=${daemon.getWwwAuthToken()}")
}
