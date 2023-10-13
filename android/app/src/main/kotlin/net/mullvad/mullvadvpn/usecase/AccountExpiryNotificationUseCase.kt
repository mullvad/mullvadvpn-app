package net.mullvad.mullvadvpn.usecase

import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.map
import net.mullvad.mullvadvpn.model.AccountExpiry
import net.mullvad.mullvadvpn.repository.InAppNotification
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManager
import net.mullvad.mullvadvpn.util.flatMapReadyConnectionOrDefault
import org.joda.time.DateTime

class AccountExpiryNotificationUseCase(
    private val serviceConnectionManager: ServiceConnectionManager,
) {
    fun notifications(): Flow<List<InAppNotification>> =
        serviceConnectionManager.connectionState
            .flatMapReadyConnectionOrDefault(flowOf(emptyList())) {
                it.container.accountDataSource.accountExpiry
                    .map { accountExpiry -> accountExpiryNotification(accountExpiry) }
                    .map(::listOfNotNull)
            }
            .distinctUntilChanged()

    private fun accountExpiryNotification(accountExpiry: AccountExpiry) =
        if (accountExpiry.isCloseToExpiring()) {
            InAppNotification.AccountExpiry(accountExpiry.date() ?: DateTime.now())
        } else null

    private fun AccountExpiry.isCloseToExpiring(): Boolean {
        val threeDaysFromNow = DateTime.now().plusDays(3)
        return this.date()?.isBefore(threeDaysFromNow) == true
    }
}
