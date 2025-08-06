package net.mullvad.mullvadvpn.usecase

import arrow.core.Either
import arrow.core.left
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
    suspend operator fun invoke(selection: Selection): Either<SelectHopError, Unit> =
        if (selection.hop.isActive) {
            selectHop(selection = selection)
        } else {
            SelectHopError.HopInactive(hop = selection.hop).left()
        }

    private suspend fun selectHop(selection: Selection): Either<SelectHopError, Unit> =
        when (selection) {
            is Selection.MultiHop -> {
                val entryConstraint = selection.hop.entry.id
                val exitConstraint = selection.hop.exit.id
                if (entryConstraint == exitConstraint) {
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
            is Selection.Entry -> {
                val locationConstraint = selection.hop.relay.id
                if (settingsRepository.settingsUpdates.value.isExit(locationConstraint)) {
                    SelectHopError.EntryAndExitSame.left()
                } else {
                    wireguardConstraintsRepository.setEntryLocation(locationConstraint).mapLeft {
                        SelectHopError.GenericError
                    }
                }
            }
            is Selection.Exit -> {
                val locationConstraint = selection.hop.relay.id
                if (
                    settingsRepository.settingsUpdates.value.multihopEnabled() &&
                        settingsRepository.settingsUpdates.value.isEntry(locationConstraint)
                ) {
                    SelectHopError.EntryAndExitSame.left()
                } else {
                    relayListRepository.updateSelectedRelayLocation(locationConstraint).mapLeft {
                        SelectHopError.GenericError
                    }
                }
            }
        }

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

    private fun Settings?.multihopEnabled(): Boolean {
        return this?.relaySettings?.relayConstraints?.wireguardConstraints?.isMultihopEnabled ==
            true
    }
}

sealed interface SelectHopError {
    data class HopInactive(val hop: Hop) : SelectHopError

    data object EntryAndExitSame : SelectHopError

    data object GenericError : SelectHopError
}

sealed class Selection {
    abstract val hop: Hop

    data class Entry(override val hop: Hop.Single<*>) : Selection()

    data class Exit(override val hop: Hop.Single<*>) : Selection()

    data class MultiHop(override val hop: Hop.Multi) : Selection()
}
