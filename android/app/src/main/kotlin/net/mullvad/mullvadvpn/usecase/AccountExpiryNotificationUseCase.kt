package net.mullvad.mullvadvpn.usecase

import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.map
import net.mullvad.mullvadvpn.lib.model.AccountData
import net.mullvad.mullvadvpn.lib.shared.AccountRepository
import net.mullvad.mullvadvpn.repository.InAppNotification
import net.mullvad.mullvadvpn.service.notifications.accountexpiry.ACCOUNT_EXPIRY_CLOSE_TO_EXPIRY_THRESHOLD_DAYS
import org.joda.time.DateTime

class AccountExpiryNotificationUseCase(private val accountRepository: AccountRepository) {
    operator fun invoke(): Flow<List<InAppNotification>> =
        accountRepository.accountData
            .map(::accountExpiryNotification)
            .map(::listOfNotNull)
            .distinctUntilChanged()

    private fun accountExpiryNotification(accountData: AccountData?) =
        if (accountData != null && accountData.expiryDate.isCloseToExpiring()) {
            InAppNotification.AccountExpiry(accountData.expiryDate)
        } else null

    private fun DateTime.isCloseToExpiring(): Boolean {
        val threeDaysFromNow =
            DateTime.now().plusDays(ACCOUNT_EXPIRY_CLOSE_TO_EXPIRY_THRESHOLD_DAYS)
        return isBefore(threeDaysFromNow)
    }
}
