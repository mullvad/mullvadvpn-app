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
import net.mullvad.mullvadvpn.lib.ipc.Event
import net.mullvad.mullvadvpn.lib.ipc.MessageHandler
import net.mullvad.mullvadvpn.lib.ipc.events
import net.mullvad.mullvadvpn.model.AccountExpiry
import net.mullvad.mullvadvpn.model.TunnelState
import net.mullvad.mullvadvpn.repository.AccountRepository
import net.mullvad.talpid.tunnel.ErrorStateCause
import org.joda.time.DateTime

class OutOfTimeUseCase(
    private val repository: AccountRepository,
    private val messageHandler: MessageHandler,
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

    // What if we already are out of time?
    private fun isTunnelBlockedBecauseOutOfTime(): Flow<Boolean> =
        messageHandler
            .events<Event.TunnelStateChange>()
            .map { it.tunnelState.isTunnelErrorStateDueToExpiredAccount() }
            .onStart { emit(false) }

    private fun TunnelState.isTunnelErrorStateDueToExpiredAccount(): Boolean {
        return ((this as? TunnelState.Error)?.errorState?.cause as? ErrorStateCause.AuthFailed)
            ?.isCausedByExpiredAccount() ?: false
    }

    private fun pastAccountExpiry(): Flow<Boolean?> =
        repository.accountExpiryState
            .flatMapLatest {
                if (it is AccountExpiry.Available) {
                    flow {
                        val millisUntilExpiry = it.expiryDateTime.millis - DateTime.now().millis
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
