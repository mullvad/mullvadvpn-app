package net.mullvad.mullvadvpn.service.notifications

import android.app.Notification
import android.app.NotificationManager
import android.app.PendingIntent
import android.content.Context
import android.content.Intent
import android.net.Uri
import kotlin.properties.Delegates.observable
import kotlinx.coroutines.delay
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.model.LoginStatus
import net.mullvad.mullvadvpn.service.MullvadDaemon
import net.mullvad.mullvadvpn.service.endpoint.AccountCache
import net.mullvad.mullvadvpn.util.Intermittent
import net.mullvad.mullvadvpn.util.JobTracker
import org.joda.time.DateTime
import org.joda.time.Duration

class AccountExpiryNotification(
    val context: Context,
    val daemon: Intermittent<MullvadDaemon>,
    val accountCache: AccountCache
) {
    companion object {
        val NOTIFICATION_ID: Int = 2
        val REMAINING_TIME_FOR_REMINDERS = Duration.standardDays(2)
        val TIME_BETWEEN_CHECKS: Long = 12 /* h */ * 60 /* min */ * 60 /* s */ * 1000 /* ms */
    }

    private val jobTracker = JobTracker()
    private val resources = context.resources

    private val buyMoreTimeUrl = resources.getString(R.string.account_url)

    private val channel = NotificationChannel(
        context,
        "mullvad_account_time",
        R.string.account_time_notification_channel_name,
        R.string.account_time_notification_channel_description,
        NotificationManager.IMPORTANCE_HIGH
    )

    var loginStatus by observable<LoginStatus?>(null) { _, oldValue, newValue ->
        if (oldValue != newValue) {
            jobTracker.newUiJob("update") { update(newValue) }
        }
    }

    init {
        accountCache.onLoginStatusChange.subscribe(this) { newStatus ->
            loginStatus = newStatus
        }
    }

    fun onDestroy() {
        accountCache.onAccountNumberChange.unsubscribe(this)
        loginStatus = null
    }

    private suspend fun update(loginStatus: LoginStatus?) {
        val remainingTime = loginStatus?.expiry?.let { expiry -> Duration(DateTime.now(), expiry) }
        val closeToExpire = remainingTime?.isShorterThan(REMAINING_TIME_FOR_REMINDERS) ?: false
        val accountIsNew = loginStatus?.isNewAccount ?: false

        if (closeToExpire && !accountIsNew) {
            val notification = build(loginStatus!!.expiry!!, remainingTime!!)

            channel.notificationManager.notify(NOTIFICATION_ID, notification)

            jobTracker.newUiJob("scheduleUpdate") { scheduleUpdate() }
        } else {
            channel.notificationManager.cancel(NOTIFICATION_ID)
            jobTracker.cancelJob("scheduleUpdate")
        }
    }

    private suspend fun scheduleUpdate() {
        delay(TIME_BETWEEN_CHECKS)
        update(loginStatus)
    }

    private suspend fun build(expiry: DateTime, remainingTime: Duration): Notification {
        val url = jobTracker.runOnBackground {
            Uri.parse("$buyMoreTimeUrl?token=${daemon.await().getWwwAuthToken()}")
        }

        val intent = Intent(Intent.ACTION_VIEW, url)
        val flags = PendingIntent.FLAG_UPDATE_CURRENT
        val pendingIntent = PendingIntent.getActivity(context, 1, intent, flags)

        return channel.buildNotification(pendingIntent, format(expiry, remainingTime))
    }

    private fun format(expiry: DateTime, remainingTime: Duration): String {
        if (remainingTime.isShorterThan(Duration.ZERO)) {
            return resources.getString(R.string.account_credit_has_expired)
        } else {
            val remainingTimeInfo = remainingTime.toPeriodTo(expiry)

            if (remainingTimeInfo.days >= 1) {
                return getRemainingText(
                    R.plurals.account_credit_expires_in_days,
                    remainingTime.standardDays.toInt()
                )
            } else if (remainingTimeInfo.hours >= 1) {
                return getRemainingText(
                    R.plurals.account_credit_expires_in_hours,
                    remainingTime.standardHours.toInt()
                )
            } else {
                return resources.getString(R.string.account_credit_expires_in_a_few_minutes)
            }
        }
    }

    private fun getRemainingText(pluralId: Int, quantity: Int): String {
        return resources.getQuantityString(pluralId, quantity, quantity)
    }
}
