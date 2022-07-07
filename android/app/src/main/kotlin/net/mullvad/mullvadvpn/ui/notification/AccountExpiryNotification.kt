package net.mullvad.mullvadvpn.ui.notification

import android.content.Context
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.util.TimeLeftFormatter
import org.joda.time.DateTime

class AccountExpiryNotification(
    context: Context,
) : InAppNotification() {
    private val timeLeftFormatter = TimeLeftFormatter(context.resources)

    init {
        status = StatusLevel.Error
        title = context.getString(R.string.account_credit_expires_soon)
    }

    fun updateAccountExpiry(expiry: DateTime?) {
        val threeDaysFromNow = DateTime.now().plusDays(3)

        if (expiry != null && expiry.isBefore(threeDaysFromNow)) {
            message = timeLeftFormatter.format(expiry)
            shouldShow = true
        } else {
            shouldShow = false
        }

        update()
    }
}
