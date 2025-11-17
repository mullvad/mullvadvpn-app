package net.mullvad.mullvadvpn.usecase

import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.zip
import net.mullvad.mullvadvpn.compose.state.MultihopRelayListType
import net.mullvad.mullvadvpn.compose.state.RelayListType
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.lib.model.Settings
import net.mullvad.mullvadvpn.relaylist.isTheSameAs
import net.mullvad.mullvadvpn.relaylist.withDescendants
import net.mullvad.mullvadvpn.repository.RelayListRepository
import net.mullvad.mullvadvpn.repository.SettingsRepository
import net.mullvad.mullvadvpn.util.isDaitaDirectOnly
import net.mullvad.mullvadvpn.util.isDaitaEnabled

class RelayItemCanBeSelectedUseCase(
    private val filteredRelayListUseCase: FilteredRelayListUseCase,
    private val hopSelectionUseCase: HopSelectionUseCase,
    private val settingsRepository: SettingsRepository,
    private val relayListRepository: RelayListRepository,
) {
    operator fun invoke() =
        invoke(MultihopRelayListType.ENTRY).zip(invoke(MultihopRelayListType.EXIT)) { entry, exit ->
            entry to exit
        }

    operator fun invoke(selectedAs: MultihopRelayListType): Flow<Set<RelayItem.Location>> =
        combine(
            relayListRepository.relayList,
            filteredRelayListUseCase(RelayListType.Multihop(selectedAs)),
            hopSelectionUseCase(),
            settingsRepository.settingsUpdates,
        ) { relayItems, filteredRelayCountries, hopSelection, settings ->
            relayItems
                .withDescendants()
                .filter { relayItem ->
                    // Item need to be active
                    if (!relayItem.active) {
                        return@filter false
                    }
                    // If entry selection, check if entry is blocked
                    if (
                        selectedAs == MultihopRelayListType.ENTRY &&
                            settings?.entrySelectionBlocked() == true
                    ) {
                        return@filter false
                    }
                    // Finally just do a normal check
                    checkValid(
                        filteredRelayCountries = filteredRelayCountries,
                        selectedRelayItem =
                            when (selectedAs) {
                                MultihopRelayListType.ENTRY -> hopSelection.entry()
                                MultihopRelayListType.EXIT -> hopSelection.exit()
                            },
                        relayItem = relayItem,
                    )
                }
                .toSet()
        }

    /**
     * Check if the relay item is in the list of the list of filtered relay countries and not the
     * same as the selected relay item
     */
    private fun checkValid(
        filteredRelayCountries: List<RelayItem.Location.Country>,
        selectedRelayItem: Constraint<RelayItem>?,
        relayItem: RelayItem,
    ): Boolean =
        when {
            filteredRelayCountries.withDescendants().none { it.id == relayItem.id } -> false
            selectedRelayItem?.getOrNull()?.isTheSameAs(relayItem) == true -> false
            else -> true
        }
}

private fun Settings.entrySelectionBlocked() = isDaitaEnabled() && !isDaitaDirectOnly()
