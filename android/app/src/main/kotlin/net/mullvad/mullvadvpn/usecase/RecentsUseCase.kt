package net.mullvad.mullvadvpn.usecase

import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.map
import net.mullvad.mullvadvpn.compose.state.RelayListType
import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.GeoLocationId
import net.mullvad.mullvadvpn.lib.model.Hop
import net.mullvad.mullvadvpn.lib.model.Recent
import net.mullvad.mullvadvpn.lib.model.Recents
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.lib.model.RelayItemId
import net.mullvad.mullvadvpn.relaylist.withDescendants
import net.mullvad.mullvadvpn.repository.SettingsRepository
import net.mullvad.mullvadvpn.usecase.customlists.FilterCustomListsRelayItemUseCase

class RecentsUseCase(
    private val customListsRelayItemUseCase: FilterCustomListsRelayItemUseCase,
    private val filteredRelayListUseCase: FilteredRelayListUseCase,
    private val settingsRepository: SettingsRepository,
) {

    operator fun invoke(): Flow<List<Hop>?> =
        combine(
            recents(),
            allEntryRelays(),
            customListsRelayItemUseCase(RelayListType.ENTRY),
            allExitRelays(),
            customListsRelayItemUseCase(RelayListType.EXIT),
        ) { recents, entryRelayList, entryCustomLists, exitRelayList, exitCustomLists ->
            recents?.mapNotNull { recent ->
                when (recent) {
                    is Recent.Multihop -> {
                        val entry = recent.entry.findItem(entryCustomLists, entryRelayList)
                        val exit = recent.exit.findItem(exitCustomLists, exitRelayList)

                        if (entry != null && exit != null) {
                            Hop.Multi(entry, exit)
                        } else {
                            null
                        }
                    }
                    is Recent.Singlehop -> {
                        val relayListItem = recent.location.findItem(exitCustomLists, exitRelayList)

                        relayListItem?.let { Hop.Single(it) }
                    }
                }
            }
        }

    private fun recents(): Flow<List<Recent>?> =
        settingsRepository.settingsUpdates.map { settings ->
            val recents = settings?.recents
            when (recents) {
                is Recents.Enabled -> {
                    recents.recents
                }

                Recents.Disabled,
                null -> null
            }
        }

    private fun allEntryRelays(): Flow<List<RelayItem.Location>> =
        filteredRelayListUseCase(RelayListType.ENTRY).distinctUntilChanged().map {
            it.withDescendants()
        }

    private fun allExitRelays(): Flow<List<RelayItem.Location>> =
        filteredRelayListUseCase(RelayListType.EXIT).distinctUntilChanged().map {
            it.withDescendants()
        }

    fun RelayItemId.findItem(
        customLists: List<RelayItem.CustomList>,
        relayList: List<RelayItem.Location>,
    ): RelayItem? =
        when (this) {
            is CustomListId -> customLists.firstOrNull { this == it.id }
            is GeoLocationId -> {
                relayList.firstOrNull { this == it.id }
            }
        }
}
