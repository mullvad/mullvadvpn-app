package net.mullvad.mullvadvpn.lib.pushnotification.worker

import android.app.Notification
import android.content.Context
import androidx.work.CoroutineWorker
import androidx.work.ForegroundInfo
import androidx.work.WorkerParameters
import co.touchlab.kermit.Logger
import java.time.Duration
import java.time.ZonedDateTime
import kotlin.getValue
import kotlinx.coroutines.withTimeoutOrNull
import net.mullvad.mullvadvpn.lib.common.serviceconnection.bindVpnService
import net.mullvad.mullvadvpn.lib.common.util.ACCOUNT_EXPIRY_CLOSE_TO_EXPIRY_THRESHOLD
import net.mullvad.mullvadvpn.lib.model.NotificationChannel
import net.mullvad.mullvadvpn.lib.pushnotification.ScheduleNotificationAlarmUseCase
import net.mullvad.mullvadvpn.lib.pushnotification.accountexpiry.AccountExpiryNotificationProvider
import net.mullvad.mullvadvpn.lib.repository.AccountRepository
import net.mullvad.mullvadvpn.lib.ui.resource.R
import org.koin.core.component.KoinComponent
import org.koin.core.component.inject

class ExpiryNotificationWorker(private val appContext: Context, workerParams: WorkerParameters) :
    CoroutineWorker(appContext, workerParams), KoinComponent {

    private val notificationProvider by inject<AccountExpiryNotificationProvider>()
    private val scheduleNotificationAlarmUseCase by inject<ScheduleNotificationAlarmUseCase>()
    private val accountRepository by inject<AccountRepository>()
    private val notificationChannel by inject<NotificationChannel.AccountUpdates>()

    override suspend fun doWork(): Result {
        // Bind to the VPN service to make sure the daemon is started
        val serviceConnection = appContext.bindVpnService()

        val expiry =
            withTimeoutOrNull(ACCOUNT_WAIT_TIMEOUT_MS) {
                // Call for an account expiry update
                accountRepository.refreshAccountData(waitForDeviceState = true)

                // Check account data
                accountRepository.accountData.value?.expiryDate
            }

        // If we get a null we should just exit and not schedule a new notification.
        // This either because we were unable to update the account data or that we are no longer
        // logged in.
        if (expiry == null) {
            Logger.e("Error! Were unable to retrieve expiry date")
            appContext.unbindService(serviceConnection)
            return Result.success()
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

        return Result.success()
    }

    override suspend fun getForegroundInfo(): ForegroundInfo {
        return ForegroundInfo(
            NOTIFICATION_ID,
            Notification.Builder(appContext, notificationChannel.id.value)
                .setSmallIcon(R.drawable.small_logo_white)
                .build(),
        )
    }

    companion object {
        private const val ACCOUNT_WAIT_TIMEOUT_MS = 5000L
        private const val NOTIFICATION_ID = 2
    }
}
