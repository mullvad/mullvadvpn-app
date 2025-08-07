package net.mullvad.mullvadvpn.usecase

import arrow.core.Either
import arrow.core.raise.either
import kotlinx.coroutines.flow.firstOrNull
import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.GeoLocationId
import net.mullvad.mullvadvpn.lib.model.Hop
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.lib.model.RelayItemId
import net.mullvad.mullvadvpn.lib.model.Settings
import net.mullvad.mullvadvpn.relaylist.getById
import net.mullvad.mullvadvpn.repository.RelayListRepository
import net.mullvad.mullvadvpn.repository.SettingsRepository
import net.mullvad.mullvadvpn.usecase.customlists.CustomListsRelayItemUseCase

class ModifyMultihopUseCase(
    private val relayListRepository: RelayListRepository,
    private val customListsRelayItemUseCase: CustomListsRelayItemUseCase,
    private val settingsRepository: SettingsRepository,
    private val selectHopUseCase: SelectHopUseCase,
) {
    suspend operator fun invoke(change: MultihopChange): Either<ModifyMultihopError, Unit> =
        either {
            val newMultihop =
                when (change) {
                    is MultihopChange.Entry -> {
                        Hop.Multi(
                            entry = change.item,
                            exit =
                                settingsRepository.settingsUpdates.value.exitLocation(
                                    relayListRepository = relayListRepository,
                                    customListsRelayItemUseCase = customListsRelayItemUseCase,
                                ) ?: raise(ModifyMultihopError.GenericError),
                        )
                    }
                    is MultihopChange.Exit -> {
                        Hop.Multi(
                            entry =
                                settingsRepository.settingsUpdates.value.entryLocation(
                                    relayListRepository = relayListRepository,
                                    customListsRelayItemUseCase = customListsRelayItemUseCase,
                                ) ?: raise(ModifyMultihopError.GenericError),
                            exit = change.item,
                        )
                    }
                }

            selectHopUseCase(newMultihop)
                .mapLeft { error ->
                    when (error) {
                        is SelectHopError.HopInactive ->
                            ModifyMultihopError.RelayItemInactive(change.item)
                        is SelectHopError.EntryAndExitSame ->
                            when (change) {
                                is MultihopChange.Entry -> ModifyMultihopError.ExitSame(change.item)
                                is MultihopChange.Exit -> ModifyMultihopError.EntrySame(change.item)
                            }
                        SelectHopError.GenericError -> ModifyMultihopError.GenericError
                    }
                }
                .bind()
        }

    private suspend fun Settings?.exitLocation(
        relayListRepository: RelayListRepository,
        customListsRelayItemUseCase: CustomListsRelayItemUseCase,
    ): RelayItem? =
        findLocation(
            relayItemId = exit(),
            relayListRepository = relayListRepository,
            customListsRelayItemUseCase = customListsRelayItemUseCase,
        )

    private suspend fun Settings?.entryLocation(
        relayListRepository: RelayListRepository,
        customListsRelayItemUseCase: CustomListsRelayItemUseCase,
    ): RelayItem? =
        findLocation(
            relayItemId = entry(),
            relayListRepository = relayListRepository,
            customListsRelayItemUseCase = customListsRelayItemUseCase,
        )

    private suspend fun findLocation(
        relayItemId: RelayItemId?,
        relayListRepository: RelayListRepository,
        customListsRelayItemUseCase: CustomListsRelayItemUseCase,
    ): RelayItem? =
        when (relayItemId) {
            is CustomListId -> customListsRelayItemUseCase().firstOrNull()?.getById(relayItemId)
            is GeoLocationId -> relayListRepository.find(relayItemId)
            else -> null
        }

    private fun Settings?.exit() = this?.relaySettings?.relayConstraints?.location?.getOrNull()

    private fun Settings?.entry() =
        this?.relaySettings?.relayConstraints?.wireguardConstraints?.entryLocation?.getOrNull()
}

sealed class MultihopChange {
    abstract val item: RelayItem

    data class Entry(override val item: RelayItem) : MultihopChange()

    data class Exit(override val item: RelayItem) : MultihopChange()
}

sealed interface ModifyMultihopError {
    data class RelayItemInactive(val relayItem: RelayItem) : ModifyMultihopError

    data class EntrySame(val relayItem: RelayItem) : ModifyMultihopError

    data class ExitSame(val relayItem: RelayItem) : ModifyMultihopError

    data object GenericError : ModifyMultihopError
}
