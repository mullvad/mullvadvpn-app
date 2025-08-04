package net.mullvad.mullvadvpn.usecase

import arrow.core.Either
import arrow.core.left
import net.mullvad.mullvadvpn.compose.state.RelayListType
import net.mullvad.mullvadvpn.lib.model.Hop
import net.mullvad.mullvadvpn.lib.model.RelayItemId
import net.mullvad.mullvadvpn.lib.model.Settings
import net.mullvad.mullvadvpn.repository.RelayListRepository
import net.mullvad.mullvadvpn.repository.SettingsRepository
import net.mullvad.mullvadvpn.repository.WireguardConstraintsRepository

class SelectHopUseCase(
    private val wireguardConstraintsRepository: WireguardConstraintsRepository,
    private val relayListRepository: RelayListRepository,
    private val settingsRepository: SettingsRepository,
) {
    suspend operator fun invoke(
        hop: Hop,
        selectedRelayListType: RelayListType,
    ): Either<SelectHopError, Unit> =
        if (hop.isActive) {
            selectHop(hop = hop, selectedRelayListType = selectedRelayListType)
        } else {
            SelectHopError.HopNotActive(hop = hop).left()
        }

    private suspend fun selectHop(
        hop: Hop,
        selectedRelayListType: RelayListType,
    ): Either<SelectHopError, Unit> =
        when (hop) {
            is Hop.Multi -> {
                val entryConstraint = hop.entry.id
                val exitConstraint = hop.exit.id
                relayListRepository.updateSelectedRelayLocationMultihop(
                    entry = entryConstraint,
                    exit = exitConstraint,
                )
            }
            is Hop.Single<*> -> {
                val locationConstraint = hop.relay.id
                when (selectedRelayListType) {
                    RelayListType.ENTRY ->
                        if (settingsRepository.settingsUpdates.value.isExit(locationConstraint)) {
                            SelectHopError.ExitBlocked.left()
                        } else {
                            wireguardConstraintsRepository.setEntryLocation(locationConstraint)
                        }
                    RelayListType.EXIT ->
                        if (settingsRepository.settingsUpdates.value.isEntry(locationConstraint)) {
                            SelectHopError.EntryBlocked.left()
                        } else {
                            relayListRepository.updateSelectedRelayLocation(locationConstraint)
                        }
                }
            }
        }.mapLeft { it as? SelectHopError ?: SelectHopError.GenericError }

    private fun Settings?.isExit(locationConstraint: RelayItemId): Boolean {
        return this?.relaySettings?.relayConstraints?.location?.getOrNull() == locationConstraint
    }

    private fun Settings?.isEntry(locationConstraint: RelayItemId): Boolean {
        return this?.relaySettings
            ?.relayConstraints
            ?.wireguardConstraints
            ?.entryLocation
            ?.getOrNull() == locationConstraint
    }
}

sealed interface SelectHopError {
    data class HopNotActive(val hop: Hop) : SelectHopError

    data object ExitBlocked : SelectHopError

    data object EntryBlocked : SelectHopError

    data object GenericError : SelectHopError
}
