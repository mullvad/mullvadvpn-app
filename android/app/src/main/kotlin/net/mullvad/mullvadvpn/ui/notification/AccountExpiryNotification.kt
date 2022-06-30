package net.mullvad.mullvadvpn.ui.notification

import android.content.Context
import kotlinx.coroutines.flow.collect
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.ui.serviceconnection.AccountRepository
import net.mullvad.mullvadvpn.ui.serviceconnection.AuthTokenCache
import net.mullvad.mullvadvpn.util.TimeLeftFormatter
import org.joda.time.DateTime

class AccountExpiryNotification(
    context: Context,
    authTokenCache: AuthTokenCache,
    private val accountRepository: AccountRepository
) : NotificationWithUrlWithToken(context, authTokenCache, R.string.account_url) {
    private val timeLeftFormatter = TimeLeftFormatter(context.resources)

    init {
        status = StatusLevel.Error
        title = context.getString(R.string.account_credit_expires_soon)
    }

    override fun onResume() {
        jobTracker.newUiJob("updateAccountExpiry") {
            accountRepository.accountExpiryState.collect { state ->
                updateAccountExpiry(state.date())
            }
        }
    }

    override fun onPause() {
        jobTracker.cancelJob("updateAccountExpiry")
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
