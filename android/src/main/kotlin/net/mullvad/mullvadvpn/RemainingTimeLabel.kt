package net.mullvad.mullvadvpn

import kotlinx.coroutines.async
import kotlinx.coroutines.launch
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope

import org.joda.time.format.DateTimeFormat
import org.joda.time.DateTime
import org.joda.time.Duration
import org.joda.time.PeriodType

import android.view.View
import android.widget.TextView

class RemainingTimeLabel(val parentActivity: MainActivity, val view: View) {
    private val accountCache = parentActivity.accountCache

    private val expiredColor = parentActivity.getColor(R.color.red)
    private val normalColor = parentActivity.getColor(R.color.white60)

    private val resources = parentActivity.resources

    private val label = view.findViewById<TextView>(R.id.remaining_time)

    private var updateJob = updateLabel()

    fun onDestroy() {
        updateJob.cancel()
    }

    private fun updateLabel() = GlobalScope.launch(Dispatchers.Main) {
        val expiry = accountCache.accountExpiry.await()

        if (expiry != null) {
            val remainingTime = Duration(DateTime.now(), expiry)

            if (remainingTime.isShorterThan(Duration.ZERO)) {
                label.setText(R.string.out_of_time)
                label.setTextColor(expiredColor)
            } else {
                val remainingTimeInfo =
                    remainingTime.toPeriodTo(expiry, PeriodType.yearMonthDayTime())

                if (remainingTimeInfo.years > 0) {
                    label.setText(getRemainingText(R.plurals.years_left, remainingTimeInfo.years))
                } else if (remainingTimeInfo.months > 0) {
                    label.setText(getRemainingText(R.plurals.months_left, remainingTimeInfo.months))
                } else if (remainingTimeInfo.days > 0) {
                    label.setText(getRemainingText(R.plurals.days_left, remainingTimeInfo.days))
                } else {
                    label.setText(R.string.less_than_a_day_left)
                }

                label.setTextColor(normalColor)
            }
        } else {
            label.text = ""
        }
    }

    private fun getRemainingText(pluralId: Int, quantity: Int): String {
        return resources.getQuantityString(pluralId, quantity, quantity)
    }
}
