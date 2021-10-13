package net.mullvad.mullvadvpn.ui.notification

import android.content.Context
import android.content.Intent
import android.net.Uri
import androidx.annotation.StringRes
import net.mullvad.mullvadvpn.ui.serviceconnection.AuthTokenCache

abstract class NotificationWithUrlWithToken(
    protected val context: Context,
    protected val authTokenCache: AuthTokenCache,
    @StringRes urlId: Int
) : InAppNotification() {
    private val url = context.getString(urlId)

    protected val openUrl: suspend () -> Unit = {
        context.startActivity(Intent(Intent.ACTION_VIEW, buildUrl()))
    }

    init {
        onClick = openUrl
        showIcon = true
    }

    private suspend fun buildUrl() = Uri.parse("$url?token=${authTokenCache.fetchAuthToken()}")
}
