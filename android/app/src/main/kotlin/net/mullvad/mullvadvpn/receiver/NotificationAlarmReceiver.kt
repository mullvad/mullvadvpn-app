package net.mullvad.mullvadvpn.receiver

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import co.touchlab.kermit.Logger
import java.time.Duration
import java.time.ZonedDateTime
import net.mullvad.mullvadvpn.service.notifications.accountexpiry.AccountExpiryNotificationProvider
import net.mullvad.mullvadvpn.usecase.ScheduleNotificationAlarmUseCase
import net.mullvad.mullvadvpn.util.goAsync
import net.mullvad.mullvadvpn.util.serializable
import org.koin.core.component.KoinComponent
import org.koin.core.component.inject

class NotificationAlarmReceiver : BroadcastReceiver(), KoinComponent {
    private val notificationProvider by inject<AccountExpiryNotificationProvider>()
    private val scheduleNotificationAlarmUseCase by inject<ScheduleNotificationAlarmUseCase>()

    override fun onReceive(context: Context?, intent: Intent?) {

        val expiry: ZonedDateTime? = intent?.serializable(ACCOUNT_EXPIRY_KEY)
        if (expiry == null) {
            Logger.e("NotificationAlarmReceiver: Intent missing account expiry")
            return
        }

        Logger.d("Account expiry alarm triggered")
        val untilExpiry = Duration.between(ZonedDateTime.now(), expiry)

        notificationProvider.showNotification(untilExpiry)

        goAsync {
            // Only schedule the next alarm if we still have time left on the account.
            if (context != null && expiry > ZonedDateTime.now()) {
                scheduleNotificationAlarmUseCase(accountExpiry = expiry, customContext = context)
            }
        }
    }

    companion object {
        const val ACCOUNT_EXPIRY_KEY: String = "account_expiry_key"
    }
}
