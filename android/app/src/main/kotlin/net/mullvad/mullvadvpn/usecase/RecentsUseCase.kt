package net.mullvad.mullvadvpn.usecase

import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.filterIsInstance
import kotlinx.coroutines.flow.map
import net.mullvad.mullvadvpn.compose.state.MultihopRelayListType
import net.mullvad.mullvadvpn.compose.state.RelayListType
import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.GeoLocationId
import net.mullvad.mullvadvpn.lib.model.Hop
import net.mullvad.mullvadvpn.lib.model.Recent
import net.mullvad.mullvadvpn.lib.model.Recents
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.lib.model.RelayItemId
import net.mullvad.mullvadvpn.relaylist.findByGeoLocationId
import net.mullvad.mullvadvpn.repository.SettingsRepository
import net.mullvad.mullvadvpn.usecase.customlists.FilterCustomListsRelayItemUseCase

class RecentsUseCase(
    private val customListsRelayItemUseCase: FilterCustomListsRelayItemUseCase,
    private val filteredRelayListUseCase: FilteredRelayListUseCase,
    private val settingsRepository: SettingsRepository,
) {

    operator fun invoke(relayListType: RelayListType): Flow<List<Hop.Single<RelayItem>>?> =
        if (relayListType is RelayListType.Multihop) {
            multiHopRecents(relayListType.multihopRelayListType)
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

    private fun multiHopRecents(
        multihopRelayListType: MultihopRelayListType
    ): Flow<List<Hop.Single<RelayItem>>?> =
        combine(
            recents().map { it?.filterIsInstance<Recent.Multihop>() },
            filteredRelayListUseCase(RelayListType.Multihop(multihopRelayListType)),
            customListsRelayItemUseCase(RelayListType.Multihop(multihopRelayListType)),
        ) { recents, relayList, customLists ->
            recents?.mapNotNull { recent ->
                val item =
                    if (multihopRelayListType == MultihopRelayListType.ENTRY) {
                            recent.entry
                        } else {
                            recent.exit
                        }
                        .findItem(customLists, relayList)
                item?.let { Hop.Single(it) }
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
