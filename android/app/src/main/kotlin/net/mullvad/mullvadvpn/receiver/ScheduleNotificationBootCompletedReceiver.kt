package net.mullvad.mullvadvpn.receiver

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import co.touchlab.kermit.Logger
import kotlinx.coroutines.runBlocking
import net.mullvad.mullvadvpn.repository.UserPreferencesRepository
import net.mullvad.mullvadvpn.usecase.ScheduleNotificationAlarmUseCase
import org.koin.core.component.KoinComponent
import org.koin.core.component.inject

class ScheduleNotificationBootCompletedReceiver : BroadcastReceiver(), KoinComponent {
    private val userPreferencesRepository by inject<UserPreferencesRepository>()
    private val scheduleNotificationAlarmUseCase by inject<ScheduleNotificationAlarmUseCase>()

    override fun onReceive(context: Context?, intent: Intent?) {
        if (intent?.action == Intent.ACTION_BOOT_COMPLETED) {
            context?.let {
                Logger.d(
                    "Scheduling notification alarm from ScheduleNotificationBootCompletedReceiver"
                )
                runBlocking { scheduleAccountExpiryNotification(context) }
            }
        }
    }

    private suspend fun scheduleAccountExpiryNotification(context: Context) {
        val expiry = userPreferencesRepository.accountExpiry() ?: return
        scheduleNotificationAlarmUseCase(context, expiry)
    }
}
