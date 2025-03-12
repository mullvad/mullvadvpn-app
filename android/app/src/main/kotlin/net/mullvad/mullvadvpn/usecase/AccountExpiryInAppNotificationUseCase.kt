package net.mullvad.mullvadvpn.usecase

import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.map
import net.mullvad.mullvadvpn.lib.model.InAppNotification
import net.mullvad.mullvadvpn.lib.shared.AccountRepository
import net.mullvad.mullvadvpn.service.notifications.accountexpiry.ACCOUNT_EXPIRY_CLOSE_TO_EXPIRY_THRESHOLD
import net.mullvad.mullvadvpn.service.notifications.accountexpiry.ACCOUNT_EXPIRY_IN_APP_NOTIFICATION_UPDATE_INTERVAL
import net.mullvad.mullvadvpn.service.notifications.accountexpiry.AccountExpiryTicker

class AccountExpiryInAppNotificationUseCase(private val accountRepository: AccountRepository) {

    @OptIn(kotlinx.coroutines.ExperimentalCoroutinesApi::class)
    operator fun invoke(): Flow<List<InAppNotification>> =
        accountRepository.accountData
            .flatMapLatest { accountData ->
                if (accountData != null) {
                    AccountExpiryTicker.tickerFlow(
                            expiry = accountData.expiryDate,
                            tickStart = ACCOUNT_EXPIRY_CLOSE_TO_EXPIRY_THRESHOLD,
                            updateInterval = { ACCOUNT_EXPIRY_IN_APP_NOTIFICATION_UPDATE_INTERVAL },
                        )
                        .map { tick ->
                            when (tick) {
                                AccountExpiryTicker.NotWithinThreshold -> emptyList()
                                is AccountExpiryTicker.Tick ->
                                    listOf(InAppNotification.AccountExpiry(tick.expiresIn))
                            }
                        }
                } else {
                    flowOf(emptyList())
                }
            }
            .distinctUntilChanged()
}
