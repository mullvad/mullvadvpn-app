package net.mullvad.mullvadvpn.usecase

import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.map
import net.mullvad.mullvadvpn.constant.ACCOUNT_EXPIRY_CLOSE_TO_EXPIRY_THRESHOLD_DAYS
import net.mullvad.mullvadvpn.model.AccountExpiry
import net.mullvad.mullvadvpn.repository.AccountRepository
import net.mullvad.mullvadvpn.repository.InAppNotification
import org.joda.time.DateTime

class AccountExpiryNotificationUseCase(
    private val accountRepository: AccountRepository,
) {
    fun notifications(): Flow<List<InAppNotification>> =
        accountRepository.accountExpiry
            .map(::accountExpiryNotification)
            .map(::listOfNotNull)
            .distinctUntilChanged()

    private fun accountExpiryNotification(accountExpiry: AccountExpiry) =
        if (accountExpiry.isCloseToExpiring()) {
            InAppNotification.AccountExpiry(accountExpiry.date() ?: DateTime.now())
        } else null

    private fun AccountExpiry.isCloseToExpiring(): Boolean {
        val threeDaysFromNow =
            DateTime.now().plusDays(ACCOUNT_EXPIRY_CLOSE_TO_EXPIRY_THRESHOLD_DAYS)
        return this.date()?.isBefore(threeDaysFromNow) == true
    }
}
