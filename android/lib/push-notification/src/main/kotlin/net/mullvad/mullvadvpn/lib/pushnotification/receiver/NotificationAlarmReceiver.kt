package net.mullvad.mullvadvpn.lib.pushnotification.receiver

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.os.Build
import androidx.core.os.UserManagerCompat
import androidx.work.Constraints
import androidx.work.NetworkType
import androidx.work.OneTimeWorkRequestBuilder
import androidx.work.OutOfQuotaPolicy
import androidx.work.WorkManager
import co.touchlab.kermit.Logger
import net.mullvad.mullvadvpn.lib.pushnotification.worker.ExpiryNotificationWorker
import org.koin.core.component.KoinComponent

class NotificationAlarmReceiver : BroadcastReceiver(), KoinComponent {

    override fun onReceive(context: Context, intent: Intent?) {
        // It is not possible to bind to a service from a notification alarm receiver so we will use
        // a worker instead.
        Logger.d("Account expiry alarm triggered")

        if (!UserManagerCompat.isUserUnlocked(context)) {
            // This should not happen, but just in case we have a check here to avoid initializing
            // work manager when the user is locked since it causes a crash
            Logger.w("User locked, skipping work request")
            return
        }

        val work =
            OneTimeWorkRequestBuilder<ExpiryNotificationWorker>()
                .apply {
                    // Setting expedited on android 12 or lower will cause the work manager to
                    // request a wake lock. We want to avoid using wakelocks so we disable expedited
                    // on android 12 and lower.
                    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
                        setExpedited(OutOfQuotaPolicy.RUN_AS_NON_EXPEDITED_WORK_REQUEST)
                    }
                }
                .setConstraints(
                    Constraints.Builder().setRequiredNetworkType(NetworkType.CONNECTED).build(),
                )
                .build()
        WorkManager.getInstance(context).enqueue(work)
        return
    }
}
