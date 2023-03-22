package net.mullvad.mullvadvpn.util

import android.content.res.Resources
import net.mullvad.mullvadvpn.R
import org.joda.time.DateTime
import org.joda.time.Duration
import org.joda.time.PeriodType

class TimeAgoFormatter(val resources: Resources) {
    private val periodType = PeriodType.standard().withMillisRemoved().withSecondsRemoved()

    fun format(instant: DateTime): String {
        val elapsedTime = Duration(instant, DateTime.now())
        val elapsedTimeInfo = elapsedTime.toPeriodTo(instant, periodType)

        if (elapsedTimeInfo.years > 0) {
            return getRemainingText(R.plurals.years_ago, elapsedTimeInfo.years)
        } else if (elapsedTimeInfo.months > 0) {
            return getRemainingText(R.plurals.months_ago, elapsedTimeInfo.months)
        } else if (elapsedTimeInfo.days > 0) {
            return getRemainingText(R.plurals.days_ago, elapsedTimeInfo.days)
        } else if (elapsedTimeInfo.hours > 0) {
            return getRemainingText(R.plurals.hours_ago, elapsedTimeInfo.hours)
        } else if (elapsedTimeInfo.minutes > 0) {
            return getRemainingText(R.plurals.minutes_ago, elapsedTimeInfo.minutes)
        } else {
            return resources.getString(R.string.less_than_a_minute_ago)
        }
    }

    private fun getRemainingText(pluralId: Int, quantity: Int): String {
        return resources.getQuantityString(pluralId, quantity, quantity)
    }
}
