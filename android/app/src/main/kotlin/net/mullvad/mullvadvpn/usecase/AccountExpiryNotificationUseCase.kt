package net.mullvad.mullvadvpn.usecase

import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.map
import net.mullvad.mullvadvpn.constant.ACCOUNT_EXPIRY_CLOSE_TO_EXPIRY_THRESHOLD_DAYS
import net.mullvad.mullvadvpn.lib.account.AccountRepository
import net.mullvad.mullvadvpn.model.AccountData
import net.mullvad.mullvadvpn.repository.InAppNotification
import org.joda.time.DateTime

class AccountExpiryNotificationUseCase(
    private val accountRepository: AccountRepository,
) {
    fun notifications(): Flow<List<InAppNotification>> =
        accountRepository.accountData
            .filterNotNull()
            .map(::accountExpiryNotification)
            .map(::listOfNotNull)
            .distinctUntilChanged()

    private fun accountExpiryNotification(accountData: AccountData) =
        if (accountData.expiryDate.isCloseToExpiring()) {
            InAppNotification.AccountExpiry(accountData.expiryDate)
        } else null

    private fun DateTime.isCloseToExpiring(): Boolean {
        val threeDaysFromNow =
            DateTime.now().plusDays(ACCOUNT_EXPIRY_CLOSE_TO_EXPIRY_THRESHOLD_DAYS)
        return isBefore(threeDaysFromNow)
    }
}
