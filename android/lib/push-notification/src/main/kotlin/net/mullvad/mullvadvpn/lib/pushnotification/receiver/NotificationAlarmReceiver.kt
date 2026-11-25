package net.mullvad.mullvadvpn.lib.pushnotification.receiver

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import co.touchlab.kermit.Logger
import java.time.Duration
import java.time.ZonedDateTime
import kotlin.time.Duration.Companion.seconds
import net.mullvad.mullvadvpn.lib.common.serviceconnection.bindVpnService
import net.mullvad.mullvadvpn.lib.common.util.ACCOUNT_EXPIRY_CLOSE_TO_EXPIRY_THRESHOLD
import net.mullvad.mullvadvpn.lib.common.util.goAsync
import net.mullvad.mullvadvpn.lib.pushnotification.ScheduleNotificationAlarmUseCase
import net.mullvad.mullvadvpn.lib.pushnotification.accountexpiry.AccountExpiryNotificationProvider
import net.mullvad.mullvadvpn.lib.repository.AccountRepository
import org.koin.core.component.KoinComponent
import org.koin.core.component.inject

class NotificationAlarmReceiver : BroadcastReceiver(), KoinComponent {

    private val notificationProvider by inject<AccountExpiryNotificationProvider>()
    private val scheduleNotificationAlarmUseCase by inject<ScheduleNotificationAlarmUseCase>()
    private val accountRepository by inject<AccountRepository>()

    override fun onReceive(context: Context, intent: Intent?) {
        Logger.d("Account expiry alarm triggered")

        goAsync { checkAndShowNotification(context.applicationContext) }
    }

    private suspend fun checkAndShowNotification(appContext: Context) {
        // Bind to the VPN service to make sure the daemon is started
        val serviceConnection = appContext.bindVpnService()

        val expiry = accountRepository.fetchAccountData(timeout = ACCOUNT_WAIT_TIMEOUT)?.expiryDate

        // If we get a null we should just exit and not schedule a new notification.
        // This either because we were unable to update the account data or that we are no longer
        // logged in.
        if (expiry == null) {
            Logger.e("Error! Were unable to retrieve expiry date")
            appContext.unbindService(serviceConnection)
            return
        }

        val untilExpiry = Duration.between(ZonedDateTime.now(), expiry)

        // Only show notification if untilExpiry is less than the threshold
        if (untilExpiry <= ACCOUNT_EXPIRY_CLOSE_TO_EXPIRY_THRESHOLD) {
            notificationProvider.showNotification(untilExpiry)
        }

        // Only schedule the next alarm if we still have time left on the account.
        if (expiry > ZonedDateTime.now()) {
            scheduleNotificationAlarmUseCase(accountExpiry = expiry, customContext = appContext)
        }

        appContext.unbindService(serviceConnection)
    }

    companion object {
        private val ACCOUNT_WAIT_TIMEOUT = 5.seconds
    }
}
