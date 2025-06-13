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

class TimeChangedReceiver : BroadcastReceiver(), KoinComponent {
    private val userPreferencesRepository by inject<UserPreferencesRepository>()
    private val scheduleNotificationAlarmUseCase by inject<ScheduleNotificationAlarmUseCase>()

    override fun onReceive(context: Context?, intent: Intent?) {
        if (
            intent?.action == Intent.ACTION_TIME_CHANGED ||
                intent?.action == Intent.ACTION_TIMEZONE_CHANGED
        ) {
            runBlocking {
                val expiry = userPreferencesRepository.accountExpiry()
                if (context != null && expiry != null) {
                    Logger.d("Scheduling notification alarm from TimeChangedReceiver")
                    scheduleNotificationAlarmUseCase(context, expiry)
                }
            }
        }
    }
}
