package net.mullvad.mullvadvpn.usecase

import arrow.core.Either
import arrow.core.left
import net.mullvad.mullvadvpn.lib.model.Hop
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.repository.RelayListRepository

class SelectHopUseCase(private val relayListRepository: RelayListRepository) {
    suspend operator fun invoke(hop: Hop): Either<SelectHopError, Unit> =
        if (hop.isActive) {
            selectHop(hop = hop)
        } else {
            SelectHopError.HopInactive(hop = hop).left()
        }

    private suspend fun selectHop(hop: Hop): Either<SelectHopError, Unit> =
        when (hop) {
            is Hop.Multi -> {
                val entryConstraint = hop.entry.id
                val exitConstraint = hop.exit.id
                if (hop.entry is RelayItem.Location.Relay && entryConstraint == exitConstraint) {
                    SelectHopError.EntryAndExitSame.left()
                } else {
                    relayListRepository
                        .updateSelectedRelayLocationMultihop(
                            entry = entryConstraint,
                            exit = exitConstraint,
                        )
                        .mapLeft { SelectHopError.GenericError }
                }
            }
            is Hop.Single<*> -> {
                val locationConstraint = hop.relay.id
                relayListRepository.updateSelectedRelayLocation(locationConstraint).mapLeft {
                    SelectHopError.GenericError
                }
            }
        }
}

sealed interface SelectHopError {
    data class HopInactive(val hop: Hop) : SelectHopError

    data object EntryAndExitSame : SelectHopError

    data object GenericError : SelectHopError
}
