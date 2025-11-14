package net.mullvad.mullvadvpn.usecase

import arrow.core.Either
import arrow.core.raise.either
import arrow.core.raise.ensure
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.repository.RelayListRepository

class SelectSinglehopUseCase(private val relayListRepository: RelayListRepository) {
    suspend operator fun invoke(item: RelayItem): Either<SelectRelayItemError, Unit> = either {
        ensure(item.active) { SelectRelayItemError.RelayInactive(relayItem = item) }
        relayListRepository.updateSelectedRelayLocation(item.id).mapLeft {
            SelectRelayItemError.GenericError
        }
    }
}

sealed interface SelectRelayItemError {
    data class RelayInactive(val relayItem: RelayItem) : SelectRelayItemError

    data object EntryAndExitSame : SelectRelayItemError

    data object GenericError : SelectRelayItemError
}
