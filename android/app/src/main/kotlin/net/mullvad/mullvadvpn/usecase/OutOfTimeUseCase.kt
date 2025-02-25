package net.mullvad.mullvadvpn.usecase

import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.ExperimentalCoroutinesApi
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
import net.mullvad.mullvadvpn.lib.common.util.millisFromNow
import net.mullvad.mullvadvpn.lib.model.ErrorStateCause
import net.mullvad.mullvadvpn.lib.model.TunnelState
import net.mullvad.mullvadvpn.lib.shared.AccountRepository
import net.mullvad.mullvadvpn.lib.shared.ConnectionProxy

class OutOfTimeUseCase(
    private val connectionProxy: ConnectionProxy,
    private val repository: AccountRepository,
    scope: CoroutineScope,
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
        connectionProxy.tunnelState
            .map { it.isTunnelErrorStateDueToExpiredAccount() }
            .onStart { emit(false) }

    private fun TunnelState.isTunnelErrorStateDueToExpiredAccount(): Boolean {
        return ((this as? TunnelState.Error)?.errorState?.cause as? ErrorStateCause.AuthFailed)
            ?.isCausedByExpiredAccount() ?: false
    }

    @OptIn(ExperimentalCoroutinesApi::class)
    private fun pastAccountExpiry(): Flow<Boolean?> =
        repository.accountData
            .flatMapLatest {
                if (it != null) {
                    flow {
                        val millisUntilExpiry = it.expiryDate.millisFromNow()
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
