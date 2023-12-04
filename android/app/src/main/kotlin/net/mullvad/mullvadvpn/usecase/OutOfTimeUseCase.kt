package net.mullvad.mullvadvpn.usecase

import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.onStart
import net.mullvad.mullvadvpn.lib.ipc.Event
import net.mullvad.mullvadvpn.lib.ipc.MessageHandler
import net.mullvad.mullvadvpn.lib.ipc.events
import net.mullvad.mullvadvpn.model.AccountExpiry
import net.mullvad.mullvadvpn.model.TunnelState
import net.mullvad.mullvadvpn.repository.AccountRepository
import net.mullvad.talpid.tunnel.ErrorStateCause
import org.joda.time.DateTime

const val accountRefreshInterval = 1000L * 60L // 1 minute

class OutOfTimeUseCase(
    private val repository: AccountRepository,
    private val messageHandler: MessageHandler
) {

    fun isOutOfTime() =
        combine(pastAccountExpiry(), isTunnelBlockedBecauseOutOfTime()) {
                accountExpiryHasPast,
                tunnelOutOfTime ->
                accountExpiryHasPast or tunnelOutOfTime
            }
            .distinctUntilChanged()

    private fun isTunnelBlockedBecauseOutOfTime() =
        messageHandler
            .events<Event.TunnelStateChange>()
            .map { it.tunnelState.isTunnelErrorStateDueToExpiredAccount() }
            .onStart { emit(false) }

    private fun TunnelState.isTunnelErrorStateDueToExpiredAccount(): Boolean {
        return ((this as? TunnelState.Error)?.errorState?.cause as? ErrorStateCause.AuthFailed)
            ?.isCausedByExpiredAccount()
            ?: false
    }

    private fun pastAccountExpiry(): Flow<Boolean> =
        combine(repository.accountExpiryState, timeFlow()) { accountExpiryState, time ->
            when (accountExpiryState) {
                is AccountExpiry.Available -> {
                    accountExpiryState.date()?.isBefore(time) ?: false
                }
                AccountExpiry.Missing -> false
            }
        }

    private fun timeFlow() = flow {
        while (true) {
            emit(DateTime.now())
            delay(accountRefreshInterval)
        }
    }
}
