package net.mullvad.mullvadvpn.ui.notification

import android.content.Context
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.service.MullvadDaemon
import net.mullvad.mullvadvpn.ui.serviceconnection.AccountCache
import net.mullvad.mullvadvpn.util.TimeLeftFormatter
import org.joda.time.DateTime

class AccountExpiryNotification(
    context: Context,
    daemon: MullvadDaemon,
    private val accountCache: AccountCache
) : NotificationWithUrlWithToken(context, daemon, R.string.account_url) {
    private val timeLeftFormatter = TimeLeftFormatter(context.resources)

    init {
        status = StatusLevel.Error
        title = context.getString(R.string.account_credit_expires_soon)
    }

    override fun onResume() {
        accountCache.onAccountExpiryChange.subscribe(this) { accountExpiry ->
            jobTracker.newUiJob("updateAccountExpiry") {
                updateAccountExpiry(accountExpiry)
            }
        }
    }

    override fun onPause() {
        accountCache.onAccountExpiryChange.unsubscribe(this)
    }

    private fun updateAccountExpiry(expiry: DateTime?) {
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
