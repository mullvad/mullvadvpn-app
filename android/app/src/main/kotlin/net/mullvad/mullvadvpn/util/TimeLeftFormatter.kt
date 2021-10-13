package net.mullvad.mullvadvpn.util

import android.content.res.Resources
import net.mullvad.mullvadvpn.R
import org.joda.time.DateTime
import org.joda.time.Duration
import org.joda.time.PeriodType

class TimeLeftFormatter(val resources: Resources) {
    fun format(accountExpiry: DateTime): String {
        val remainingTime = Duration(DateTime.now(), accountExpiry)

        return format(accountExpiry, remainingTime)
    }

    fun format(accountExpiry: DateTime, remainingTime: Duration): String {
        if (remainingTime.isShorterThan(Duration.ZERO)) {
            return resources.getString(R.string.out_of_time)
        } else {
            val remainingTimeInfo =
                remainingTime.toPeriodTo(accountExpiry, PeriodType.yearMonthDayTime())

            if (remainingTimeInfo.years > 0) {
                return getRemainingText(R.plurals.years_left, remainingTimeInfo.years)
            } else if (remainingTimeInfo.months >= 3) {
                return getRemainingText(R.plurals.months_left, remainingTimeInfo.months)
            } else if (remainingTimeInfo.months > 0 || remainingTimeInfo.days >= 1) {
                return getRemainingText(R.plurals.days_left, remainingTime.standardDays.toInt())
            } else {
                return resources.getString(R.string.less_than_a_day_left)
            }
        }
    }

    private fun getRemainingText(pluralId: Int, quantity: Int): String {
        return resources.getQuantityString(pluralId, quantity, quantity)
    }
}
