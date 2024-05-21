package net.mullvad.mullvadvpn.usecase

import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.onStart
import kotlinx.coroutines.flow.stateIn
import net.mullvad.mullvadvpn.lib.account.AccountRepository
import net.mullvad.mullvadvpn.lib.daemon.grpc.ManagementService
import net.mullvad.mullvadvpn.model.ErrorStateCause
import net.mullvad.mullvadvpn.model.TunnelState
import org.joda.time.DateTime

class OutOfTimeUseCase(
    private val managementService: ManagementService,
    private val repository: net.mullvad.mullvadvpn.lib.account.AccountRepository,
    scope: CoroutineScope
) {

    val isOutOfTime: StateFlow<Boolean?> =
        combine(pastAccountExpiry(), isTunnelBlockedBecauseOutOfTime()) {
                accountExpiryHasPast,
                tunnelOutOfTime ->
                reduce(accountExpiryHasPast, tunnelOutOfTime)
            }
            .stateIn(scope, SharingStarted.Eagerly, null)

    private fun reduce(vararg outOfTimeProperty: Boolean?): Boolean? =
        when {
            // If any advertises as out of time
            outOfTimeProperty.any { it == true } -> true
            // If all advertise as not out of time
            outOfTimeProperty.all { it == false } -> false
            // If some are unknown
            else -> null
        }

    private fun isTunnelBlockedBecauseOutOfTime(): Flow<Boolean> =
        managementService.tunnelState
            .map { it.isTunnelErrorStateDueToExpiredAccount() }
            .onStart { emit(false) }

    private fun TunnelState.isTunnelErrorStateDueToExpiredAccount(): Boolean {
        return ((this as? TunnelState.Error)?.errorState?.cause as? ErrorStateCause.AuthFailed)
            ?.isCausedByExpiredAccount() ?: false
    }

    private fun pastAccountExpiry(): Flow<Boolean?> =
        repository.accountData
            .flatMapLatest {
                if (it != null) {
                    flow {
                        val millisUntilExpiry = it.expiryDate.millis - DateTime.now().millis
                        if (millisUntilExpiry > 0) {
                            emit(false)
                            delay(millisUntilExpiry)
                            emit(true)
                        } else {
                            emit(true)
                        }
                    }
                } else {
                    flowOf<Boolean?>(null)
                }
            }
            .distinctUntilChanged()
}
