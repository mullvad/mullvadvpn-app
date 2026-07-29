package net.mullvad.mullvadvpn.lib.usecase

import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.map
import net.mullvad.mullvadvpn.lib.common.util.relaylist.findByGeoLocationId
import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.EntryRecent
import net.mullvad.mullvadvpn.lib.model.GeoLocationId
import net.mullvad.mullvadvpn.lib.model.MultihopRelayListType
import net.mullvad.mullvadvpn.lib.model.RecentItem
import net.mullvad.mullvadvpn.lib.model.Recents
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.lib.model.RelayItemId
import net.mullvad.mullvadvpn.lib.model.RelayListType
import net.mullvad.mullvadvpn.lib.repository.SettingsRepository
import net.mullvad.mullvadvpn.lib.usecase.customlists.FilterCustomListsRelayItemUseCase

class RecentsUseCase(
    private val customListsRelayItemUseCase: FilterCustomListsRelayItemUseCase,
    private val filteredRelayListUseCase: FilteredRelayListUseCase,
    private val settingsRepository: SettingsRepository,
) {

    operator fun invoke(relayListType: RelayListType): Flow<List<RecentItem>?> =
        when (relayListType) {
            is RelayListType.Multihop -> multihopRecents(relayListType.multihopRelayListType)
            RelayListType.Single -> singlehopRecents()
        }

    private fun singlehopRecents(): Flow<List<RecentItem>?> =
        combine(
            recents(),
            filteredRelayListUseCase(RelayListType.Single),
            customListsRelayItemUseCase(RelayListType.Single),
        ) { recents, relayList, customLists ->
            recents?.exit?.mapNotNull { recent ->
                recent.location.findItem(customLists, relayList)
            }
        }

    private fun multihopRecents(
        multihopRelayListType: MultihopRelayListType
    ): Flow<List<RecentItem>?> =
        combine(
            recents(),
            filteredRelayListUseCase(RelayListType.Multihop(multihopRelayListType)),
            customListsRelayItemUseCase(RelayListType.Multihop(multihopRelayListType)),
        ) { recents, relayList, customLists ->
            val enabled = recents ?: return@combine null

            when (multihopRelayListType) {
                MultihopRelayListType.ENTRY ->
                    enabled.entry.mapNotNull { recent ->
                        when (recent) {
                            EntryRecent.Automatic -> RecentItem.Automatic
                            is EntryRecent.Location ->
                                recent.location.findItem(customLists, relayList)
                        }
                    }
                MultihopRelayListType.EXIT ->
                    enabled.exit.mapNotNull { recent ->
                        recent.location.findItem(customLists, relayList)
                    }
            }
        }

    private fun recents(): Flow<Recents.Enabled?> =
        settingsRepository.settingsUpdates.map { settings ->
            when (val recents = settings?.recents) {
                is Recents.Enabled -> recents
                Recents.Disabled,
                null -> null
            }
        }

    private fun RelayItemId.findItem(
        customLists: List<RelayItem.CustomList>,
        relayList: List<RelayItem.Location.Country>,
    ): RecentItem.Relay? =
        when (this) {
            is CustomListId ->
                customLists.firstOrNull { this == it.id && it.hasChildren }.toRecent()
            is GeoLocationId -> relayList.findByGeoLocationId(this).toRecent()
        }

    private fun RelayItem?.toRecent(): RecentItem.Relay? = this?.let { RecentItem.Relay(it) }
}
