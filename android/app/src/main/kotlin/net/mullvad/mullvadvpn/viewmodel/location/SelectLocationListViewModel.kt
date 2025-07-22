package net.mullvad.mullvadvpn.viewmodel.location

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.stateIn
import net.mullvad.mullvadvpn.compose.state.RelayListType
import net.mullvad.mullvadvpn.compose.state.SelectLocationListUiState
import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.GeoLocationId
import net.mullvad.mullvadvpn.lib.model.RelayItemId
import net.mullvad.mullvadvpn.repository.RelayListRepository
import net.mullvad.mullvadvpn.repository.SettingsRepository
import net.mullvad.mullvadvpn.repository.WireguardConstraintsRepository
import net.mullvad.mullvadvpn.usecase.FilteredRelayListUseCase
import net.mullvad.mullvadvpn.usecase.RecentsUseCase
import net.mullvad.mullvadvpn.usecase.SelectedLocationUseCase
import net.mullvad.mullvadvpn.usecase.customlists.CustomListsRelayItemUseCase
import net.mullvad.mullvadvpn.usecase.customlists.FilterCustomListsRelayItemUseCase
import net.mullvad.mullvadvpn.util.Lce

class SelectLocationListViewModel(
    private val relayListType: RelayListType,
    private val filteredRelayListUseCase: FilteredRelayListUseCase,
    private val filteredCustomListRelayItemsUseCase: FilterCustomListsRelayItemUseCase,
    private val selectedLocationUseCase: SelectedLocationUseCase,
    private val wireguardConstraintsRepository: WireguardConstraintsRepository,
    private val relayListRepository: RelayListRepository,
    private val recentsUseCase: RecentsUseCase,
    private val settingsRepository: SettingsRepository,
    customListsRelayItemUseCase: CustomListsRelayItemUseCase,
) : ViewModel() {
    private val _expandedItems: MutableStateFlow<Set<String>> =
        MutableStateFlow(initialExpand(initialSelection()))

    val uiState: StateFlow<Lce<Unit, SelectLocationListUiState, Unit>> =
        combine(
                relayListItems(),
                customListsRelayItemUseCase(),
                settingsRepository.settingsUpdates,
            ) { relayListItems, customLists, settings ->
                if (relayListType == RelayListType.ENTRY && settings?.entryBlocked() == true) {
                    Lce.Error(Unit)
                } else {
                    Lce.Content(
                        SelectLocationListUiState(
                            relayListItems = relayListItems,
                            customLists = customLists,
                        )
                    )
                }
            }
            .stateIn(viewModelScope, SharingStarted.Lazily, Lce.Loading(Unit))

    fun onToggleExpand(item: RelayItemId, parent: CustomListId? = null, expand: Boolean) {
        _expandedItems.onToggleExpandSet(item, parent, expand)
    }

    private fun relayListItems() =
        combine(
            filteredRelayListUseCase(relayListType = relayListType),
            filteredCustomListRelayItemsUseCase(relayListType = relayListType),
            recentsUseCase(),
            selectedLocationUseCase(),
            _expandedItems,
        ) { relayCountries, customLists, recents, selectedItem, expandedItems ->
            // If we have no locations we have an empty relay list
            // and we should show an error
            if (relayCountries.isEmpty()) {
                emptyLocationsRelayListItems(
                    relayListType = relayListType,
                    customLists = customLists,
                    selectedByThisEntryExitList =
                        selectedItem.selectedByThisEntryExitList(relayListType),
                    selectedByOtherEntryExitList =
                        selectedItem.selectedByOtherEntryExitList(relayListType, customLists),
                    expandedItems = expandedItems,
                )
            } else {
                relayListItems(
                    relayCountries = relayCountries,
                    relayListType = relayListType,
                    customLists = customLists,
                    recents = recents,
                    selectedItem = selectedItem,
                    selectedByThisEntryExitList =
                        selectedItem.selectedByThisEntryExitList(relayListType),
                    selectedByOtherEntryExitList =
                        selectedItem.selectedByOtherEntryExitList(relayListType, customLists),
                    expandedItems = expandedItems,
                    isEntryBlocked =
                        settingsRepository.settingsUpdates.value?.entryBlocked() == true,
                )
            }
        }

    private fun initialExpand(item: RelayItemId?): Set<String> = buildSet {
        when (item) {
            is GeoLocationId.City -> {
                add(item.country.code)
            }
            is GeoLocationId.Hostname -> {
                add(item.country.code)
                add(item.city.code)
            }
            is CustomListId,
            is GeoLocationId.Country,
            null -> {
                /* No expands */
            }
        }
    }

    private fun initialSelection() =
        when (relayListType) {
            RelayListType.ENTRY ->
                wireguardConstraintsRepository.wireguardConstraints.value?.entryLocation
            RelayListType.EXIT -> relayListRepository.selectedLocation.value
        }?.getOrNull()
}
