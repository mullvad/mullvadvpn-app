package net.mullvad.mullvadvpn.usecase

import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.map
import net.mullvad.mullvadvpn.compose.state.RelayListType
import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.GeoLocationId
import net.mullvad.mullvadvpn.lib.model.Hop
import net.mullvad.mullvadvpn.lib.model.Recent
import net.mullvad.mullvadvpn.lib.model.Recents
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.relaylist.withDescendants
import net.mullvad.mullvadvpn.repository.SettingsRepository
import net.mullvad.mullvadvpn.usecase.customlists.FilterCustomListsRelayItemUseCase

class RecentsUseCase(
    private val customListsRelayItemUseCase: FilterCustomListsRelayItemUseCase,
    private val filteredRelayListUseCase: FilteredRelayListUseCase,
    private val settingsRepository: SettingsRepository,
) {

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

    operator fun invoke(): Flow<List<Hop>?> =
        combine(
            recents(),
            filteredRelayListUseCase(RelayListType.ENTRY),
            customListsRelayItemUseCase(RelayListType.ENTRY),
            filteredRelayListUseCase(RelayListType.EXIT),
            customListsRelayItemUseCase(RelayListType.EXIT),
        ) { recents, entryRelayList, entryCustomLists, exitRelayList, exitCustomLists ->
            recents?.mapNotNull { recent ->
                when (recent) {
                    is Recent.Multihop -> {
                        val entry: RelayItem? =
                            when (recent.entry) {
                                is CustomListId ->
                                    entryCustomLists.firstOrNull { recent.entry == it.id }
                                is GeoLocationId ->
                                    entryRelayList.withDescendants().firstOrNull {
                                        recent.entry == it.id
                                    }
                            }
                        val exit: RelayItem? =
                            when (recent.exit) {
                                is CustomListId ->
                                    exitCustomLists.firstOrNull { recent.exit == it.id }
                                is GeoLocationId ->
                                    exitRelayList.withDescendants().firstOrNull {
                                        recent.exit == it.id
                                    }
                            }

                        if (entry != null && exit != null) {
                            Hop.Multi(entry, exit)
                        } else {
                            null
                        }
                    }
                    is Recent.Singlehop -> {
                        val relayListItem: RelayItem? =
                            when (recent.location) {
                                is CustomListId ->
                                    exitCustomLists.firstOrNull { recent.location == it.id }
                                is GeoLocationId ->
                                    exitRelayList.withDescendants().firstOrNull {
                                        recent.location == it.id
                                    }
                            }
                        if (relayListItem != null) {
                            Hop.Single(relayListItem)
                        } else {
                            null
                        }
                    }
                }
            }
        }
}
