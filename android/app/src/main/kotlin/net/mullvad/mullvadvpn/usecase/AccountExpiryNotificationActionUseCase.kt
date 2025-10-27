package net.mullvad.mullvadvpn.usecase

import java.time.ZonedDateTime
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.filter
import kotlinx.coroutines.flow.flattenConcat
import kotlinx.coroutines.flow.flow
import net.mullvad.mullvadvpn.lib.daemon.grpc.ManagementService
import net.mullvad.mullvadvpn.lib.model.DeviceState
import net.mullvad.mullvadvpn.lib.repository.AccountRepository
import net.mullvad.mullvadvpn.service.notifications.accountexpiry.ACCOUNT_EXPIRY_CLOSE_TO_EXPIRY_THRESHOLD

sealed interface NotificationAction {
    data object CancelExisting : NotificationAction

    data class ScheduleAlarm(val alarmTime: ZonedDateTime) : NotificationAction
}

class AccountExpiryNotificationActionUseCase(
    private val accountRepository: AccountRepository,
    private val managementService: ManagementService,
) {
    @OptIn(ExperimentalCoroutinesApi::class)
    operator fun invoke(): Flow<NotificationAction> =
        combine(managementService.deviceState, accountRepository.accountData) {
                deviceState,
                accountData ->
                flow {
                    when (deviceState) {
                        is DeviceState.LoggedIn -> {
                            // There are cases where the current device's account number isn't the
                            // same as the account data device number. This can happen when logging
                            // out of one account and logging in to another (the deviceState will
                            // update before the new accountData is available).
                            if (deviceState.accountNumber == accountData?.accountNumber) {
                                if (shouldCancelExisting(accountData.expiryDate)) {
                                    emit(NotificationAction.CancelExisting)
                                }
                                emit(NotificationAction.ScheduleAlarm(accountData.expiryDate))
                            }
                        }

                        DeviceState.LoggedOut,
                        DeviceState.Revoked -> emit(NotificationAction.CancelExisting)
                    }
                }
            }
            .flattenConcat()
            .filter { !accountRepository.isNewAccount.value }
            .distinctUntilChanged()

    private fun shouldCancelExisting(expiry: ZonedDateTime): Boolean {
        val expiryTimeIsAfterThreshold =
            expiry.isAfter(ZonedDateTime.now().plus(ACCOUNT_EXPIRY_CLOSE_TO_EXPIRY_THRESHOLD))

        return expiryTimeIsAfterThreshold || accountRepository.isNewAccount.value
    }
}
