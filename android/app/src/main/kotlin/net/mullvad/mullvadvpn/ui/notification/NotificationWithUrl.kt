package net.mullvad.mullvadvpn.ui.notification

import android.content.Context
import android.content.Intent
import android.content.Intent.FLAG_ACTIVITY_NEW_TASK
import android.net.Uri

abstract class NotificationWithUrl(protected val context: Context, urlId: Int) :
    InAppNotification() {
    private val url = Uri.parse(context.getString(urlId))

    protected val openUrl: suspend () -> Unit = {
        val intent = Intent(Intent.ACTION_VIEW, url).apply { flags = FLAG_ACTIVITY_NEW_TASK }
        context.startActivity(intent)
    }

    init {
        onClick = openUrl
        showIcon = true
    }
}
