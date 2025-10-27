package net.mullvad.mullvadvpn.usecase.inappnotification

import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.map
import net.mullvad.mullvadvpn.lib.model.InAppNotification
import net.mullvad.mullvadvpn.lib.repository.AccountRepository
import net.mullvad.mullvadvpn.service.notifications.accountexpiry.ACCOUNT_EXPIRY_CLOSE_TO_EXPIRY_THRESHOLD
import net.mullvad.mullvadvpn.service.notifications.accountexpiry.ACCOUNT_EXPIRY_NOTIFICATION_UPDATE_INTERVAL
import net.mullvad.mullvadvpn.service.notifications.accountexpiry.InAppAccountExpiryTicker

class AccountExpiryInAppNotificationUseCase(private val accountRepository: AccountRepository) :
    InAppNotificationUseCase {

    @OptIn(ExperimentalCoroutinesApi::class)
    override operator fun invoke(): Flow<InAppNotification?> =
        accountRepository.accountData
            .flatMapLatest { accountData ->
                if (accountData != null) {
                    InAppAccountExpiryTicker.tickerFlow(
                            expiry = accountData.expiryDate,
                            tickStart = ACCOUNT_EXPIRY_CLOSE_TO_EXPIRY_THRESHOLD,
                            updateInterval = { ACCOUNT_EXPIRY_NOTIFICATION_UPDATE_INTERVAL },
                        )
                        .map { tick ->
                            when (tick) {
                                InAppAccountExpiryTicker.NotWithinThreshold -> null
                                is InAppAccountExpiryTicker.Tick ->
                                    InAppNotification.AccountExpiry(tick.expiresIn)
                            }
                        }
                } else {
                    flowOf(null)
                }
            }
            .distinctUntilChanged()
}
