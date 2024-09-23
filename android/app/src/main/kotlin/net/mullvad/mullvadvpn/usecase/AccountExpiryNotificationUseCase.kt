package net.mullvad.mullvadvpn.usecase

import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.map
import net.mullvad.mullvadvpn.constant.ACCOUNT_EXPIRY_CLOSE_TO_EXPIRY_THRESHOLD
import net.mullvad.mullvadvpn.constant.ACCOUNT_EXPIRY_IN_APP_NOTIFICATION_UPDATE_INTERVAL
import net.mullvad.mullvadvpn.lib.shared.AccountRepository
import net.mullvad.mullvadvpn.repository.InAppNotification
import org.joda.time.DateTime
import org.joda.time.Duration

class AccountExpiryNotificationUseCase(private val accountRepository: AccountRepository) {

    operator fun invoke(): Flow<List<InAppNotification>> =
        accountRepository.accountData
            .flatMapLatest { accountData ->
                if (accountData != null) {
                    flow {
                        val expiry = accountData.expiryDate

                        Duration(DateTime.now(), expiry).let { expiresIn ->
                            if (expiresIn.isShorterThan(Duration.ZERO)) {
                                // has expired
                                return@flow
                            }
                            delayUntilNotificationThreshold(expiresIn)
                        }

                        while (true) {
                            emit(InAppNotification.AccountExpiry(expiry))

                            val expiresIn = Duration(DateTime.now(), expiry)
                            if (
                                expiresIn.isLongerThan(
                                    ACCOUNT_EXPIRY_IN_APP_NOTIFICATION_UPDATE_INTERVAL
                                )
                            ) {
                                delay(ACCOUNT_EXPIRY_IN_APP_NOTIFICATION_UPDATE_INTERVAL.millis)
                            } else {
                                break
                            }
                        }
                    }
                } else {
                    flowOf<InAppNotification?>(null)
                }
            }
            .map(::listOfNotNull)

    private suspend fun delayUntilNotificationThreshold(expiresIn: Duration) {
        if (expiresIn.isLongerThan(ACCOUNT_EXPIRY_CLOSE_TO_EXPIRY_THRESHOLD)) {
            val durationToThreshold = expiresIn.minus(ACCOUNT_EXPIRY_CLOSE_TO_EXPIRY_THRESHOLD)
            delay(durationToThreshold.millis)
        }
    }
}
