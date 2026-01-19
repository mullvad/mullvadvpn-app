package net.mullvad.mullvadvpn.worker

import android.app.Notification
import android.content.Context
import android.content.Context.BIND_AUTO_CREATE
import android.content.Intent
import android.content.ServiceConnection
import android.content.pm.ServiceInfo
import android.os.Build
import androidx.work.CoroutineWorker
import androidx.work.ForegroundInfo
import androidx.work.WorkerParameters
import co.touchlab.kermit.Logger
import java.time.Duration
import java.time.ZonedDateTime
import kotlin.getValue
import kotlinx.coroutines.withTimeoutOrNull
import net.mullvad.mullvadvpn.lib.common.constant.VPN_SERVICE_CLASS
import net.mullvad.mullvadvpn.lib.model.NotificationChannel
import net.mullvad.mullvadvpn.lib.repository.AccountRepository
import net.mullvad.mullvadvpn.service.R
import net.mullvad.mullvadvpn.service.notifications.accountexpiry.ACCOUNT_EXPIRY_CLOSE_TO_EXPIRY_THRESHOLD
import net.mullvad.mullvadvpn.service.notifications.accountexpiry.AccountExpiryNotificationProvider
import net.mullvad.mullvadvpn.ui.serviceconnection.EmptyServiceConnection
import net.mullvad.mullvadvpn.usecase.ScheduleNotificationAlarmUseCase
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
        val serviceConnection = bindVpnService(appContext)

        val expiry =
            withTimeoutOrNull(ACCOUNT_WAIT_TIMEOUT_MS) {
                // Call for an account expiry update
                accountRepository.refreshAccountData()

                // Check account data
                accountRepository.accountData.value?.expiryDate
            }

        // If we get a null we should just exist and not schedule a new notification.
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

    private fun bindVpnService(context: Context): ServiceConnection {
        val serviceConnection = EmptyServiceConnection()
        val intent = Intent().apply { setClassName(context.packageName, VPN_SERVICE_CLASS) }
        // We set BIND_AUTO_CREATE so that the service is started if it is not already running
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.UPSIDE_DOWN_CAKE) {
            context.bindService(
                intent,
                serviceConnection,
                ServiceInfo.FOREGROUND_SERVICE_TYPE_SYSTEM_EXEMPTED or BIND_AUTO_CREATE,
            )
        } else {
            context.bindService(intent, serviceConnection, BIND_AUTO_CREATE)
        }
        return serviceConnection
    }

    companion object {
        private const val ACCOUNT_WAIT_TIMEOUT_MS = 5000L
        private const val NOTIFICATION_ID = 2
    }
}
