package net.mullvad.mullvadvpn.service.notifications.accountexpiry

import java.time.ZonedDateTime
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.map
import net.mullvad.mullvadvpn.lib.model.DeviceState
import net.mullvad.mullvadvpn.lib.model.Notification
import net.mullvad.mullvadvpn.lib.model.NotificationChannelId
import net.mullvad.mullvadvpn.lib.model.NotificationId
import net.mullvad.mullvadvpn.lib.model.NotificationUpdate
import net.mullvad.mullvadvpn.lib.shared.AccountRepository
import net.mullvad.mullvadvpn.lib.shared.DeviceRepository
import net.mullvad.mullvadvpn.service.constant.IS_PLAY_BUILD
import net.mullvad.mullvadvpn.service.notifications.NotificationProvider

class AccountExpiryNotificationProvider(
    private val channelId: NotificationChannelId,
    private val accountRepository: AccountRepository,
    deviceRepository: DeviceRepository,
) : NotificationProvider<Notification.AccountExpiry> {
    @Suppress("MagicNumber") private val notificationId = NotificationId(3)

    @OptIn(ExperimentalCoroutinesApi::class)
    override val notifications: Flow<NotificationUpdate<Notification.AccountExpiry>> =
        combine(
                deviceRepository.deviceState.filterNotNull(),
                accountRepository.accountData.filterNotNull(),
                accountRepository.isNewAccount,
            ) { deviceState, accountData, isNewAccount ->
                Triple(deviceState, accountData, isNewAccount)
            }
            .flatMapLatest { (deviceState, accountData, isNewAccount) ->
                val expiry = accountData.expiryDate

                if (isNewAccount || deviceState !is DeviceState.LoggedIn) {
                    flowOf(NotificationUpdate.Cancel(notificationId))
                } else {
                    accountExpiryNotificationFlow(expiry)
                }
            }

    private fun accountExpiryNotificationFlow(
        expiryDate: ZonedDateTime
    ): Flow<NotificationUpdate<Notification.AccountExpiry>> =
        AccountExpiryTicker.tickerFlow(
                expiry = expiryDate,
                tickStart = ACCOUNT_EXPIRY_CLOSE_TO_EXPIRY_THRESHOLD,
                updateInterval = { ACCOUNT_EXPIRY_SYSTEM_NOTIFICATION_UPDATE_INTERVAL },
            )
            .map { expiryTick ->
                when (expiryTick) {
                    AccountExpiryTicker.NotWithinThreshold ->
                        NotificationUpdate.Cancel(notificationId)
                    is AccountExpiryTicker.Tick -> {
                        val notification =
                            Notification.AccountExpiry(
                                channelId = channelId,
                                actions = emptyList(),
                                websiteAuthToken =
                                    if (!IS_PLAY_BUILD) accountRepository.getWebsiteAuthToken()
                                    else null,
                                durationUntilExpiry = expiryTick.expiresIn,
                            )
                        NotificationUpdate.Notify(notificationId, notification)
                    }
                }
            }
}
