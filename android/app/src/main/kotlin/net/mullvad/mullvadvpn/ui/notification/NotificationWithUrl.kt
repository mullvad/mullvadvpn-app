package net.mullvad.mullvadvpn.ui.notification

import android.content.Context
import android.content.Intent
import android.net.Uri

abstract class NotificationWithUrl(
    protected val context: Context,
    urlId: Int
) : InAppNotification() {
    private val url = Uri.parse(context.getString(urlId))

    protected val openUrl: suspend () -> Unit = {
        context.startActivity(Intent(Intent.ACTION_VIEW, url))
    }

    init {
        onClick = openUrl
        showIcon = true
    }
}
