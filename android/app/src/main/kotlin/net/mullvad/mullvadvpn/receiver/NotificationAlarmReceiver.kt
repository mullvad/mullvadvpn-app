package net.mullvad.mullvadvpn.receiver

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import androidx.work.Constraints
import androidx.work.NetworkType
import androidx.work.OneTimeWorkRequestBuilder
import androidx.work.OutOfQuotaPolicy
import androidx.work.WorkManager
import co.touchlab.kermit.Logger
import net.mullvad.mullvadvpn.worker.ExpiryNotificationWorker
import org.koin.core.component.KoinComponent

class NotificationAlarmReceiver : BroadcastReceiver(), KoinComponent {

    override fun onReceive(context: Context, intent: Intent?) {
        // It is not possible to bind to a service from a notification alarm receiver so we will use
        // a worker instead.
        Logger.d("Account expiry alarm triggered")

        val work =
            OneTimeWorkRequestBuilder<ExpiryNotificationWorker>()
                .setExpedited(OutOfQuotaPolicy.RUN_AS_NON_EXPEDITED_WORK_REQUEST)
                .setConstraints(
                    Constraints.Builder().setRequiredNetworkType(NetworkType.CONNECTED).build()
                )
                .build()
        WorkManager.getInstance(context).enqueue(work)
        return
    }
}
