package net.mullvad.mullvadvpn.usecase

import android.app.AlarmManager
import android.app.PendingIntent
import android.content.Context
import android.content.Intent
import co.touchlab.kermit.Logger
import java.time.ZoneOffset
import java.time.ZonedDateTime
import net.mullvad.mullvadvpn.receiver.NotificationAlarmReceiver
import net.mullvad.mullvadvpn.repository.UserPreferencesRepository
import net.mullvad.mullvadvpn.service.notifications.accountexpiry.accountExpiryNotificationTriggerAt

class ScheduleNotificationAlarmUseCase(
    private val applicationContext: Context,
    private val userPreferencesRepository: UserPreferencesRepository,
) {
    suspend operator fun invoke(accountExpiry: ZonedDateTime?, customContext: Context? = null) {
        val context = customContext ?: applicationContext
        val alarmManager = context.getSystemService(AlarmManager::class.java) ?: return

        cancelExisting(context, alarmManager)

        if (accountExpiry == null) {
            userPreferencesRepository.clearAccountExpiry()
            return
        }

        val triggerAt =
            accountExpiryNotificationTriggerAt(now = ZonedDateTime.now(), expiry = accountExpiry)
        val triggerAtMillis = triggerAt.toInstant().toEpochMilli()

        val intent = alarmIntent(context, accountExpiry)
        alarmManager.set(AlarmManager.RTC, triggerAtMillis, intent)

        // Change to UTC to avoid leaking the user's time zone in the logs
        Logger.d(
            "Scheduling next account expiry alarm for ${triggerAt.withZoneSameInstant(ZoneOffset.UTC)}"
        )
        userPreferencesRepository.setAccountExpiry(accountExpiry)
    }

    private fun alarmIntent(context: Context, accountExpiry: ZonedDateTime): PendingIntent =
        Intent(context, NotificationAlarmReceiver::class.java).let { intent ->
            intent.putExtra(NotificationAlarmReceiver.ACCOUNT_EXPIRY_KEY, accountExpiry)
            PendingIntent.getBroadcast(
                context,
                ALARM_INTENT_REQUEST_CODE,
                intent,
                PendingIntent.FLAG_UPDATE_CURRENT + PendingIntent.FLAG_IMMUTABLE,
            )
        }

    private fun cancelExisting(context: Context, alarmManager: AlarmManager) {
        existingAlarmIntent(context)?.let { pendingIntent ->
            alarmManager.cancel(pendingIntent)
            Logger.d("Cancelled existing account expiry alarm")
        }
    }

    private fun existingAlarmIntent(context: Context): PendingIntent? =
        PendingIntent.getBroadcast(
            context,
            ALARM_INTENT_REQUEST_CODE,
            Intent(context, NotificationAlarmReceiver::class.java),
            PendingIntent.FLAG_UPDATE_CURRENT +
                PendingIntent.FLAG_IMMUTABLE +
                PendingIntent.FLAG_NO_CREATE,
        )
}

private const val ALARM_INTENT_REQUEST_CODE = 0
