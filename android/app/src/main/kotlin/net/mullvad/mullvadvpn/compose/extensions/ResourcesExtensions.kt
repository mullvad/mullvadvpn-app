package net.mullvad.mullvadvpn.compose.extensions

import android.content.res.Resources
import net.mullvad.mullvadvpn.R
import org.joda.time.DateTime
import org.joda.time.Duration
import org.joda.time.PeriodType

fun Resources.getExpiryQuantityString(accountExpiry: DateTime): String {
    val remainingTime = Duration(DateTime.now(), accountExpiry)

    return getExpiryQuantityString(this, accountExpiry, remainingTime)
}

private fun getExpiryQuantityString(
    resources: Resources,
    accountExpiry: DateTime,
    remainingTime: Duration
): String {
    if (remainingTime.isShorterThan(Duration.ZERO)) {
        return resources.getString(R.string.out_of_time)
    } else {
        val remainingTimeInfo =
            remainingTime.toPeriodTo(accountExpiry, PeriodType.yearMonthDayTime())

        return if (remainingTimeInfo.years > 0) {
            getRemainingText(resources, R.plurals.years_left, remainingTimeInfo.years)
        } else if (remainingTimeInfo.months >= 3) {
            getRemainingText(resources, R.plurals.months_left, remainingTimeInfo.months)
        } else if (remainingTimeInfo.months > 0 || remainingTimeInfo.days >= 1) {
            getRemainingText(resources, R.plurals.days_left, remainingTime.standardDays.toInt())
        } else {
            resources.getString(R.string.less_than_a_day_left)
        }
    }
}

private fun getRemainingText(resources: Resources, pluralId: Int, quantity: Int): String {
    return resources.getQuantityString(pluralId, quantity, quantity)
}
