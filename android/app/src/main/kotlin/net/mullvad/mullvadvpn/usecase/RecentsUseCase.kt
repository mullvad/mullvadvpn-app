package net.mullvad.mullvadvpn.usecase

import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.map
import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.GeoLocationId
import net.mullvad.mullvadvpn.lib.model.Hop
import net.mullvad.mullvadvpn.lib.model.MultihopRelayListType
import net.mullvad.mullvadvpn.lib.model.Recent
import net.mullvad.mullvadvpn.lib.model.Recents
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.lib.model.RelayItemId
import net.mullvad.mullvadvpn.lib.model.RelayListType
import net.mullvad.mullvadvpn.relaylist.findByGeoLocationId
import net.mullvad.mullvadvpn.repository.SettingsRepository
import net.mullvad.mullvadvpn.usecase.customlists.FilterCustomListsRelayItemUseCase

class RecentsUseCase(
    private val customListsRelayItemUseCase: FilterCustomListsRelayItemUseCase,
    private val filteredRelayListUseCase: FilteredRelayListUseCase,
    private val settingsRepository: SettingsRepository,
) {

    operator fun invoke(isMultihop: Boolean): Flow<List<Hop>?> =
        if (isMultihop) {
            multiHopRecents()
        } else {
            singleHopRecents()
        }

    private fun singleHopRecents(): Flow<List<Hop.Single<RelayItem>>?> =
        combine(
            recents().map { it?.filterIsInstance<Recent.Singlehop>() },
            filteredRelayListUseCase(RelayListType.Single),
            customListsRelayItemUseCase(RelayListType.Single),
        ) { recents, relayList, customList ->
            recents?.mapNotNull { recent ->
                val relayListItem = recent.location.findItem(customList, relayList)

                relayListItem?.let { Hop.Single(it) }
            }
        }

    private fun multiHopRecents(): Flow<List<Hop.Multi>?> =
        combine(
            recents().map { it?.filterIsInstance<Recent.Multihop>() },
            filteredRelayListUseCase(RelayListType.Multihop(MultihopRelayListType.ENTRY)),
            customListsRelayItemUseCase(RelayListType.Multihop(MultihopRelayListType.ENTRY)),
            filteredRelayListUseCase(RelayListType.Multihop(MultihopRelayListType.EXIT)),
            customListsRelayItemUseCase(RelayListType.Multihop(MultihopRelayListType.EXIT)),
        ) { recents, entryRelayList, entryCustomLists, exitRelayList, exitCustomLists ->
            recents?.mapNotNull { recent ->
                val entry = recent.entry.findItem(entryCustomLists, entryRelayList)
                val exit = recent.exit.findItem(exitCustomLists, exitRelayList)

                if (entry != null && exit != null) {
                    Hop.Multi(entry, exit)
                } else {
                    null
                }
            }
        }

    private fun recents(): Flow<List<Recent>?> =
        settingsRepository.settingsUpdates.map { settings ->
            val recents = settings?.recents
            when (recents) {
                is Recents.Enabled -> recents.recents
                Recents.Disabled,
                null -> null
            }
        }

    private fun RelayItemId.findItem(
        customLists: List<RelayItem.CustomList>,
        relayList: List<RelayItem.Location.Country>,
    ): RelayItem? =
        when (this) {
            is CustomListId -> customLists.firstOrNull { this == it.id && it.hasChildren }
            is GeoLocationId -> relayList.findByGeoLocationId(this)
        }
}
