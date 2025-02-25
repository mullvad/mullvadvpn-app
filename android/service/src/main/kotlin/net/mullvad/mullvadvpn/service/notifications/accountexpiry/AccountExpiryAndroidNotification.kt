package net.mullvad.mullvadvpn.service.notifications.accountexpiry

import android.app.PendingIntent
import android.content.Context
import android.content.Intent
import android.content.res.Resources
import androidx.core.app.NotificationCompat
import java.time.Duration
import net.mullvad.mullvadvpn.lib.common.constant.MAIN_ACTIVITY_CLASS
import net.mullvad.mullvadvpn.lib.common.util.SdkUtils
import net.mullvad.mullvadvpn.lib.common.util.createAccountUri
import net.mullvad.mullvadvpn.lib.model.Notification
import net.mullvad.mullvadvpn.service.R

internal fun Notification.AccountExpiry.toNotification(context: Context) =
    NotificationCompat.Builder(context, channelId.value)
        .setContentIntent(contentIntent(context))
        .setContentTitle(context.resources.contentTitle(durationUntilExpiry))
        .setSmallIcon(R.drawable.small_logo_white)
        .setOngoing(ongoing)
        .setOnlyAlertOnce(true)
        .setVisibility(NotificationCompat.VISIBILITY_PRIVATE)
        .build()

private fun Notification.AccountExpiry.contentIntent(context: Context): PendingIntent {

    val intent =
        if (websiteAuthToken == null) {
            Intent().apply {
                setClassName(context.packageName, MAIN_ACTIVITY_CLASS)
                flags = Intent.FLAG_ACTIVITY_CLEAR_TOP or Intent.FLAG_ACTIVITY_SINGLE_TOP
                action = Intent.ACTION_MAIN
            }
        } else {
            val uri = createAccountUri(context.getString(R.string.account_url), websiteAuthToken)
            Intent(Intent.ACTION_VIEW, uri)
        }
    return PendingIntent.getActivity(context, 1, intent, SdkUtils.getSupportedPendingIntentFlags())
}

private fun Resources.contentTitle(remainingTime: Duration): String =
    when {
        remainingTime <= Duration.ZERO -> {
            getString(R.string.account_credit_has_expired)
        }
        remainingTime.toDays() >= 1 -> {
            getRemainingText(
                R.plurals.account_credit_expires_in_days,
                remainingTime.toDays().toInt(),
            )
        }
        remainingTime.toHours() >= 1 -> {
            getRemainingText(
                R.plurals.account_credit_expires_in_hours,
                remainingTime.toHours().toInt(),
            )
        }
        else -> getString(R.string.account_credit_expires_in_a_few_minutes)
    }

private fun Resources.getRemainingText(pluralId: Int, quantity: Int): String {
    return getQuantityString(pluralId, quantity, quantity)
}
