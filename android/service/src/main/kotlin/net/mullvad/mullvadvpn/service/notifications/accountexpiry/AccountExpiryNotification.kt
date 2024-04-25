package net.mullvad.mullvadvpn.service.notifications.accountexpiry

import android.content.Context
import net.mullvad.mullvadvpn.lib.daemon.grpc.ManagementService
import net.mullvad.mullvadvpn.service.R
import org.joda.time.DateTime
import org.joda.time.Duration

class AccountExpiryNotification(val context: Context, val managementService: ManagementService) {

    private val resources = context.resources

    private val buyMoreTimeUrl = resources.getString(R.string.account_url)

    //    var accountExpiry by
    //        observable<AccountData>(AccountData.Missing) { _, oldValue, newValue ->
    //            if (oldValue != newValue) {
    //                jobTracker.newUiJob("update") { update(newValue) }
    //            }
    //        }

    init {
        // accountCache.onAccountExpiryChange.subscribe(this) { expiry -> accountExpiry = expiry }
    }

    fun onDestroy() {
        // accountCache.onAccountExpiryChange.unsubscribe(this)
    }

    // Suppressing since the permission check is done by calling a common util in another module.
    //    @SuppressLint("MissingPermission")
    //    private suspend fun update(accountData: AccountData?) {
    //        val durationUntilExpiry = accountData?.expiryDate?.remainingTime()
    //
    //        if (/*accountCache.isNewAccount.not() &&*/ durationUntilExpiry?.isCloseToExpiry() ==
    // true) {
    //            if (context.isNotificationPermissionMissing().not()) {
    //                val notification = build(expiryDate, durationUntilExpiry)
    //                channel.notificationManager.notify(NOTIFICATION_ID, notification)
    //            }
    //            jobTracker.newUiJob("scheduleUpdate") { scheduleUpdate() }
    //        } else {
    //            channel.notificationManager.cancel(NOTIFICATION_ID)
    //            jobTracker.cancelJob("scheduleUpdate")
    //        }
    //    }
    //
    //    private fun DateTime.remainingTime(): Duration {
    //        return Duration(DateTime.now(), this)
    //    }

    private fun Duration.isCloseToExpiry(): Boolean {
        return isShorterThan(REMAINING_TIME_FOR_REMINDERS)
    }
    //
    //    private suspend fun scheduleUpdate() {
    //        delay(TIME_BETWEEN_CHECKS)
    //        update(accountExpiry)
    //    }

    //    private suspend fun build(expiry: DateTime, remainingTime: Duration): Notification {
    //        val url =
    //            jobTracker.runOnBackground {
    //                TODO("Fetch api token from gRPC")
    //                Uri.parse("$buyMoreTimeUrl?token=TODO()}")
    //            }
    //        val intent =
    //            if (IS_PLAY_BUILD) {
    //                Intent().apply {
    //                    setClassName(context.packageName, MAIN_ACTIVITY_CLASS)
    //                    flags = Intent.FLAG_ACTIVITY_CLEAR_TOP or Intent.FLAG_ACTIVITY_SINGLE_TOP
    //                    action = Intent.ACTION_MAIN
    //                }
    //            } else {
    //                Intent(Intent.ACTION_VIEW, url)
    //            }
    //        val pendingIntent =
    //            PendingIntent.getActivity(context, 1, intent,
    // SdkUtils.getSupportedPendingIntentFlags())
    //
    //        return channel.buildNotification(pendingIntent, format(expiry, remainingTime))
    //    }
    //
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

    companion object {
        const val NOTIFICATION_ID: Int = 2
        val REMAINING_TIME_FOR_REMINDERS = Duration.standardDays(2)
        const val TIME_BETWEEN_CHECKS: Long = 12 /* h */ * 60 /* min */ * 60 /* s */ * 1000 /* ms */
    }
}
