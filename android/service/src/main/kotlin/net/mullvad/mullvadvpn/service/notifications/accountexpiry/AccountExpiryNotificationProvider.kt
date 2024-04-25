package net.mullvad.mullvadvpn.service.notifications.accountexpiry

import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.mapNotNull
import net.mullvad.mullvadvpn.lib.account.AccountRepository
import net.mullvad.mullvadvpn.model.ChannelId
import net.mullvad.mullvadvpn.model.Notification
import net.mullvad.mullvadvpn.service.notifications.NotificationProvider
import org.joda.time.DateTime
import org.joda.time.Duration

class AccountExpiryNotificationProvider(
    channelId: ChannelId,
    accountRepository: AccountRepository,
) : NotificationProvider {
    @OptIn(ExperimentalCoroutinesApi::class)
    override val notifications: Flow<Notification> =
        accountRepository.isNewAccount
            .flatMapLatest { isNewAccount ->
                if (isNewAccount) {
                    flowOf()
                } else {
                    flow {
                        while (true) {
                            // TODO do we get all the updates we need? We won't post new update if
                            // user redeems a one day voucher? Maybe needs more logic
                            emit(accountRepository.accountData.value)
                            delay(TIME_BETWEEN_CHECKS)
                            // Trigger new fetch of account data
                            accountRepository.getAccountData()
                        }
                    }
                }
            }
            .filterNotNull()
            .mapNotNull {
                val durationUntilExpiry = it.expiryDate.remainingTime()

                if (durationUntilExpiry.isCloseToExpiry()) {
                    Notification.AccountExpiry(
                        channelId = channelId,
                        actions = emptyList(),
                        durationUntilExpiry = durationUntilExpiry,
                        isPlayBuild = false
                    )
                } else {
                    null
                }
            }

    private fun DateTime.remainingTime(): Duration {
        return Duration(DateTime.now(), this)
    }

    private fun Duration.isCloseToExpiry(): Boolean {
        return isShorterThan(REMAINING_TIME_FOR_REMINDERS)
    }

    companion object {
        private val REMAINING_TIME_FOR_REMINDERS = Duration.standardDays(2)

        private const val TIME_BETWEEN_CHECKS: Long =
            12 /* h */ * 60 /* min */ * 60 /* s */ * 1000 /* ms */
    }
}
