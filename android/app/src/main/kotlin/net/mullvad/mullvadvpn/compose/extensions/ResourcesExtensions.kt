package net.mullvad.mullvadvpn.compose.extensions

import android.content.res.Resources
import net.mullvad.mullvadvpn.R
import org.joda.time.Period

fun Resources.getExpiryQuantityString(accountExpiry: Period): String {
    return if (accountExpiry.isNegative() || accountExpiry == Period.ZERO) {
        getString(R.string.out_of_time)
    } else if (accountExpiry.years > 0) {
        getRemainingText(this, R.plurals.years_left, accountExpiry.years)
    } else if (accountExpiry.months >= 3) {
        getRemainingText(this, R.plurals.months_left, accountExpiry.months)
    } else if (accountExpiry.months > 0 || accountExpiry.days >= 1) {
        getRemainingText(this, R.plurals.days_left, accountExpiry.days)
    } else {
        getString(R.string.less_than_a_day_left)
    }
}

private fun getRemainingText(resources: Resources, pluralId: Int, quantity: Int): String {
    return resources.getQuantityString(pluralId, quantity, quantity)
}

fun Period.isNegative() =
    normalizedStandard().let {
        it.years < 0 ||
            it.months < 0 ||
            it.weeks < 0 ||
            it.days < 0 ||
            it.minutes < 0 ||
            it.seconds < 0 ||
            it.millis < 0
    }
