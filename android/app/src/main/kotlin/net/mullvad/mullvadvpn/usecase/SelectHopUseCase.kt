package net.mullvad.mullvadvpn.usecase

import arrow.core.Either
import arrow.core.left
import net.mullvad.mullvadvpn.lib.model.Hop
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.lib.model.RelayItemId
import net.mullvad.mullvadvpn.lib.model.Settings
import net.mullvad.mullvadvpn.repository.RelayListRepository
import net.mullvad.mullvadvpn.repository.SettingsRepository
import net.mullvad.mullvadvpn.repository.WireguardConstraintsRepository

class ModifyMultihopUseCase(
    private val wireguardConstraintsRepository: WireguardConstraintsRepository,
    private val relayListRepository: RelayListRepository,
) {
//    suspend operator fun invoke(relayItem: RelayItem, isExit: Boolean): Either<Any, Unit> =
    suspend operator fun invoke(change: MultihopChange): Either<Any, Unit> {
        // fetch current constraints -> (entry, exit)

        val newMultihop = when(change) {
            is MultihopChange.Entry -> {
//                val entryConstraint = change.item.id
                TODO()
            }
            is MultihopChange.Exit -> {
//                val exitConstraint = change.item.id

                TODO()
            }
        }

    TODO()
//        selectHopUseCase(newMultihop).mapLeft { error ->
//            when (error) {
//                is SelectHopError.HopInactive -> error
//                is SelectHopError.EntryAndExitSame -> error
//                SelectHopError.GenericError -> Any()
//            }
//        }
    }
}

sealed class MultihopChange {
    data class Entry(val item: RelayItem) : MultihopChange()
    data class Exit(val item: RelayItem) : MultihopChange()
}

class SelectHopUseCase(
    private val wireguardConstraintsRepository: WireguardConstraintsRepository,
    private val relayListRepository: RelayListRepository,
    private val settingsRepository: SettingsRepository,
) {
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
