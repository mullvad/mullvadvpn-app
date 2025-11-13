package net.mullvad.mullvadvpn.usecase

import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.zip
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.GeoLocationId
import net.mullvad.mullvadvpn.lib.model.MultihopRelayListType
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.lib.model.RelayListType
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
    operator fun invoke(relayListType: RelayListType) =
        when (relayListType) {
            is RelayListType.Multihop ->
                validEntries(selectedAs = relayListType.multihopRelayListType.other()).map {
                    when (relayListType.multihopRelayListType) {
                        MultihopRelayListType.ENTRY -> ValidSelection.OnlyExit(exitIds = it)
                        MultihopRelayListType.EXIT -> ValidSelection.OnlyEntry(entryIds = it)
                    }
                }
            RelayListType.Single ->
                validEntries(MultihopRelayListType.ENTRY).zip(
                    validEntries(MultihopRelayListType.EXIT)
                ) { entries, exits ->
                    ValidSelection.Both(entryIds = entries, exitIds = exits)
                }
        }

    private fun validEntries(selectedAs: MultihopRelayListType): Flow<Set<GeoLocationId>> =
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
                    // If exit selection, check if entry is blocked
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
                                MultihopRelayListType.EXIT -> hopSelection.entry()
                                MultihopRelayListType.ENTRY -> hopSelection.exit()
                            },
                        relayItem = relayItem,
                    )
                }
                .map { it.id }
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

    private fun Settings.entrySelectionBlocked() = isDaitaEnabled() && !isDaitaDirectOnly()

    private fun MultihopRelayListType.other() =
        when (this) {
            MultihopRelayListType.ENTRY -> MultihopRelayListType.EXIT
            MultihopRelayListType.EXIT -> MultihopRelayListType.ENTRY
        }
}

sealed interface ValidSelection {
    val entryIds: Set<GeoLocationId>?
    val exitIds: Set<GeoLocationId>?

    data class OnlyEntry(override val entryIds: Set<GeoLocationId>) : ValidSelection {
        override val exitIds: Set<GeoLocationId>? = null
    }

    data class OnlyExit(override val exitIds: Set<GeoLocationId>) : ValidSelection {
        override val entryIds: Set<GeoLocationId>? = null
    }

    data class Both(
        override val entryIds: Set<GeoLocationId>,
        override val exitIds: Set<GeoLocationId>,
    ) : ValidSelection
}
