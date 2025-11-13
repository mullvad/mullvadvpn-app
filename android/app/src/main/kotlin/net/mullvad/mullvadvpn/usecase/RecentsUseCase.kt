package net.mullvad.mullvadvpn.usecase

import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.map
import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.GeoLocationId
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

    operator fun invoke(relayListType: RelayListType): Flow<List<RelayItem>?> =
        when (relayListType) {
            is RelayListType.Multihop -> multihopRecents(relayListType.multihopRelayListType)
            RelayListType.Single -> singlehopRecents()
        }

    private fun singlehopRecents(): Flow<List<RelayItem>?> =
        combine(
            recents().map { it?.filterIsInstance<Recent.Singlehop>() },
            filteredRelayListUseCase(RelayListType.Single),
            customListsRelayItemUseCase(RelayListType.Single),
        ) { recents, relayList, customList ->
            recents?.mapNotNull { recent -> recent.location.findItem(customList, relayList) }
        }

    private fun multihopRecents(
        multihopRelayListType: MultihopRelayListType
    ): Flow<List<RelayItem>?> =
        combine(
            recents().map { it?.filterIsInstance<Recent.Multihop>() },
            filteredRelayListUseCase(RelayListType.Multihop(multihopRelayListType)),
            customListsRelayItemUseCase(RelayListType.Multihop(multihopRelayListType)),
        ) { recents, relayList, customLists ->
            recents?.mapNotNull { recent ->
                recent.getBy(multihopRelayListType).findItem(customLists, relayList)
            }
        }

    private fun recents(): Flow<List<Recent>?> =
        settingsRepository.settingsUpdates.map { settings ->
            when (val recents = settings?.recents) {
                is Recents.Enabled -> recents.recents
                Recents.Disabled,
                null -> null
            }
        }

    private fun Recent.Multihop.getBy(multihopListType: MultihopRelayListType) =
        when (multihopListType) {
            MultihopRelayListType.ENTRY -> entry
            MultihopRelayListType.EXIT -> exit
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
