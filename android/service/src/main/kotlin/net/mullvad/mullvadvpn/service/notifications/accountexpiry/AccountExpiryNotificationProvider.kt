package net.mullvad.mullvadvpn.service.notifications.accountexpiry

import android.util.Log
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.mapNotNull
import net.mullvad.mullvadvpn.lib.daemon.grpc.ManagementService
import net.mullvad.mullvadvpn.model.AccountData
import net.mullvad.mullvadvpn.model.ChannelId
import net.mullvad.mullvadvpn.model.DeviceState
import net.mullvad.mullvadvpn.model.Notification
import net.mullvad.mullvadvpn.service.notifications.NotificationProvider
import org.joda.time.DateTime
import org.joda.time.Duration

class AccountExpiryNotificationProvider(
    channelId: ChannelId,
    managementService: ManagementService
) : NotificationProvider {
    @OptIn(ExperimentalCoroutinesApi::class)

    // TODO Should observe from AccountRepository so we get new update e.g if they redeem a single day voucher
    override val notifications: Flow<Notification> =
        managementService.deviceState
            .map {
                when (it) {
                    is DeviceState.LoggedIn -> it.accountToken
                    DeviceState.LoggedOut,
                    DeviceState.Revoked,
                    null -> null
                }
            }
            .flatMapLatest { accountToken ->
                if (accountToken == null) {
                    flowOf(null)
                } else {
                    flow<AccountData?> {
                        while (true) {
                            managementService
                                .getAccountData(accountToken)
                                .fold(
                                    { Log.d("AccountExpiryNotificationProvider", "Error: $it") },
                                    { emit(it) }
                                )
                            delay(TIME_BETWEEN_CHECKS)
                        }
                    }
                }
            }
            .filterNotNull()
            .mapNotNull {
                val durationUntilExpiry = it.expiryDate.remainingTime()

                // TODO Does not handle if it is a new account
                if (/*accountCache.isNewAccount.not() &&*/ durationUntilExpiry.isCloseToExpiry()) {
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
