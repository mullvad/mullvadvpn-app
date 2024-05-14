package net.mullvad.mullvadvpn.service.notifications.accountexpiry

import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.filterNotNull
import net.mullvad.mullvadvpn.lib.account.AccountRepository
import net.mullvad.mullvadvpn.model.ChannelId
import net.mullvad.mullvadvpn.model.Notification
import net.mullvad.mullvadvpn.service.constant.IS_PLAY_BUILD
import net.mullvad.mullvadvpn.service.notifications.NotificationProvider
import org.joda.time.DateTime
import org.joda.time.Duration

class AccountExpiryNotificationProvider(
    channelId: ChannelId,
    accountRepository: AccountRepository,
) : NotificationProvider {
    override val notifications: Flow<Notification> =
        combine(accountRepository.accountData.filterNotNull(), accountRepository.isNewAccount) {
                accountData,
                isNewAccount ->
                val durationUntilExpiry = accountData.expiryDate.remainingTime()

                val notification =
                    Notification.AccountExpiry(
                        channelId = channelId,
                        actions = emptyList(),
                        wwwAuthToken =
                            if (!IS_PLAY_BUILD) accountRepository.getWwwAuthToken() else null,
                        durationUntilExpiry = durationUntilExpiry,
                        isPlayBuild = false
                    )
                if (!isNewAccount && durationUntilExpiry.isCloseToExpiry()) {
                    notification
                } else {
                    notification.cancel()
                }
            }
            .filterNotNull()

    private fun DateTime.remainingTime(): Duration {
        return Duration(DateTime.now(), this)
    }

    private fun Duration.isCloseToExpiry(): Boolean {
        return isShorterThan(REMAINING_TIME_FOR_REMINDERS)
    }

    companion object {
        private val REMAINING_TIME_FOR_REMINDERS = Duration.standardDays(2)
    }
}
