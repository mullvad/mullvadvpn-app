package net.mullvad.mullvadvpn.compose.extensions

import android.content.res.Resources
import net.mullvad.mullvadvpn.R
import org.joda.time.DateTime
import org.joda.time.Duration
import org.joda.time.Period

fun Resources.getExpiryQuantityString(accountExpiry: Duration): String {
    val expiryPeriod = Period(DateTime.now(), accountExpiry)
    return if (accountExpiry.millis <= 0) {
        getString(R.string.out_of_time)
    } else if (expiryPeriod.years > 0) {
        getRemainingText(this, R.plurals.years_left, expiryPeriod.years)
    } else if (expiryPeriod.months >= 3) {
        getRemainingText(this, R.plurals.months_left, expiryPeriod.months)
    } else if (expiryPeriod.months > 0 || expiryPeriod.days >= 1) {
        getRemainingText(this, R.plurals.days_left, expiryPeriod.days)
    } else {
        getString(R.string.less_than_a_day_left)
    }
}

private fun getRemainingText(resources: Resources, pluralId: Int, quantity: Int): String {
    return resources.getQuantityString(pluralId, quantity, quantity)
}
