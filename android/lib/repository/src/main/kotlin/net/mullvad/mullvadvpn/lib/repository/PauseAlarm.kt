package net.mullvad.mullvadvpn.lib.repository

import android.annotation.SuppressLint
import android.app.AlarmManager
import android.app.PendingIntent
import android.content.Context
import android.content.Intent
import android.os.Build
import java.time.Instant
import net.mullvad.mullvadvpn.lib.common.constant.KEY_RESUME_ACTION
import net.mullvad.mullvadvpn.lib.common.constant.VPN_SERVICE_CLASS
import net.mullvad.mullvadvpn.lib.common.util.getSupportedPendingIntentFlags

interface PauseAlarm {
    fun schedule(resumeAt: Instant, onAlarm: () -> Unit)

    fun cancel()
}

// The alarm starts MullvadVpnService with KEY_RESUME_ACTION, so that the tunnel is reconnected even
// if the process was killed during the pause. It uses the wall clock, like the countdown that is
// shown to the user, and is allowed to fire in Doze.
//
// Exact alarms require the SCHEDULE_EXACT_ALARM permission on Android 12+ unless the app is exempt
// from battery optimizations. Without it the alarm can be delivered late, so an exact alarm with a
// listener is set as well. That one does not need the permission, but it is postponed in Doze and
// only fires while the process is alive.
class AlarmManagerPauseAlarm(private val context: Context) : PauseAlarm {
    private val alarmManager = context.getSystemService(AlarmManager::class.java)
    private var listener: AlarmManager.OnAlarmListener? = null

    override fun schedule(resumeAt: Instant, onAlarm: () -> Unit) {
        cancel()
        val triggerAtMillis = resumeAt.toEpochMilli()
        if (canScheduleExactAlarms()) {
            setExactResumeAlarm(triggerAtMillis)
        } else {
            alarmManager.setAndAllowWhileIdle(
                AlarmManager.RTC_WAKEUP,
                triggerAtMillis,
                resumeIntent(),
            )
            val listener = AlarmManager.OnAlarmListener { onAlarm() }
            alarmManager.setExact(
                AlarmManager.RTC_WAKEUP,
                triggerAtMillis,
                ALARM_TAG,
                listener,
                null,
            )
            this.listener = listener
        }
    }

    private fun canScheduleExactAlarms() =
        Build.VERSION.SDK_INT < Build.VERSION_CODES.S || alarmManager.canScheduleExactAlarms()

    // Lint cannot see that this is only called when canScheduleExactAlarms() returns true.
    @SuppressLint("MissingPermission")
    private fun setExactResumeAlarm(triggerAtMillis: Long) {
        alarmManager.setExactAndAllowWhileIdle(
            AlarmManager.RTC_WAKEUP,
            triggerAtMillis,
            resumeIntent(),
        )
    }

    override fun cancel() {
        alarmManager.cancel(resumeIntent())
        listener?.let { alarmManager.cancel(it) }
        listener = null
    }

    private fun resumeIntent(): PendingIntent =
        PendingIntent.getService(
            context,
            0,
            Intent(KEY_RESUME_ACTION).setClassName(context.packageName, VPN_SERVICE_CLASS),
            getSupportedPendingIntentFlags(),
        )

    companion object {
        private const val ALARM_TAG = "mullvad:pause"
    }
}
