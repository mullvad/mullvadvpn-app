package net.mullvad.mullvadvpn.lib.repository

import android.app.AlarmManager
import android.content.Context
import android.os.SystemClock
import kotlin.coroutines.resume
import kotlin.time.Duration
import kotlinx.coroutines.suspendCancellableCoroutine

fun interface PauseTimer {
    suspend fun await(duration: Duration)
}

// Uses an elapsed realtime alarm instead of delay(), since delay() does not count the time the
// device spends in deep sleep. An alarm with a listener does not need the SCHEDULE_EXACT_ALARM
// permission, but it only fires as long as the process is alive. Doze can still postpone it to the
// next maintenance window.
class AlarmPauseTimer(private val context: Context) : PauseTimer {
    override suspend fun await(duration: Duration) {
        val alarmManager = context.getSystemService(AlarmManager::class.java)
        suspendCancellableCoroutine { continuation ->
            val listener = AlarmManager.OnAlarmListener { continuation.resume(Unit) }
            alarmManager.setExact(
                AlarmManager.ELAPSED_REALTIME_WAKEUP,
                SystemClock.elapsedRealtime() + duration.inWholeMilliseconds,
                ALARM_TAG,
                listener,
                null,
            )
            continuation.invokeOnCancellation { alarmManager.cancel(listener) }
        }
    }

    companion object {
        private const val ALARM_TAG = "mullvad:pause"
    }
}
