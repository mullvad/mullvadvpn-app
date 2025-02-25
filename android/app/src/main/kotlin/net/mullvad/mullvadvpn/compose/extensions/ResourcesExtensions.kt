package net.mullvad.mullvadvpn.compose.extensions

import android.content.res.Resources
import java.time.Duration
import net.mullvad.mullvadvpn.R

private const val DAYS_IN_STANDARD_YEAR = 365

fun Resources.getExpiryQuantityString(accountExpiry: Duration): String {
    val days = accountExpiry.toDays().toInt()
    val years = (accountExpiry.toDays() / DAYS_IN_STANDARD_YEAR).toInt()

    return if (accountExpiry.toMillis() <= 0) {
        getString(R.string.out_of_time)
    } else if (years > 1) {
        getRemainingText(this, R.plurals.years_left, years)
    } else if (days >= 1) {
        getRemainingText(this, R.plurals.days_left, days)
    } else {
        getString(R.string.less_than_a_day_left)
    }
}

private fun getRemainingText(resources: Resources, pluralId: Int, quantity: Int): String {
    return resources.getQuantityString(pluralId, quantity, quantity)
}
