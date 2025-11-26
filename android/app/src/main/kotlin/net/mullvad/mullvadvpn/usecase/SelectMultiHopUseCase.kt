package net.mullvad.mullvadvpn.usecase

import arrow.core.Either
import arrow.core.raise.context.ensureNotNull
import arrow.core.raise.either
import arrow.core.raise.ensure
import arrow.core.raise.ensureNotNull
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.relaylist.isTheSameAs
import net.mullvad.mullvadvpn.repository.RelayListRepository

class SelectMultiHopUseCase(private val relayListRepository: RelayListRepository) {
    suspend operator fun invoke(
        entry: RelayItem?,
        exit: RelayItem,
    ): Either<SelectRelayItemError, Unit> = either {
        ensureNotNull(entry) { SelectRelayItemError.GenericError }
        ensure(entry.active) { SelectRelayItemError.RelayInactive(entry) }
        ensure(exit.active) { SelectRelayItemError.RelayInactive(exit) }
        ensure(!entry.isTheSameAs(exit)) { SelectRelayItemError.EntryAndExitSame }
        relayListRepository
            .updateSelectedRelayLocationMultihop(entry = entry.id, exit = exit.id)
            .mapLeft { SelectRelayItemError.GenericError }
    }
}
