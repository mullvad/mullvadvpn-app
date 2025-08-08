package net.mullvad.mullvadvpn.usecase

import arrow.core.Either
import arrow.core.raise.either
import arrow.core.raise.ensure
import net.mullvad.mullvadvpn.lib.model.Hop
import net.mullvad.mullvadvpn.relaylist.isTheSameAs
import net.mullvad.mullvadvpn.repository.RelayListRepository

class SelectHopUseCase(private val relayListRepository: RelayListRepository) {
    suspend operator fun invoke(hop: Hop): Either<SelectHopError, Unit> = either {
        ensure(hop.isActive) { SelectHopError.HopInactive(hop = hop) }
        when (hop) {
            is Hop.Multi -> {
                ensure(!hop.entry.isTheSameAs(hop.exit)) { SelectHopError.EntryAndExitSame }
                relayListRepository
                    .updateSelectedRelayLocationMultihop(entry = hop.entry.id, exit = hop.exit.id)
                    .mapLeft { SelectHopError.GenericError }
            }
            is Hop.Single<*> -> {
                relayListRepository.updateSelectedRelayLocation(hop.relay.id).mapLeft {
                    SelectHopError.GenericError
                }
            }
        }
    }
}

sealed interface SelectHopError {
    data class HopInactive(val hop: Hop) : SelectHopError

    data object EntryAndExitSame : SelectHopError

    data object GenericError : SelectHopError
}
