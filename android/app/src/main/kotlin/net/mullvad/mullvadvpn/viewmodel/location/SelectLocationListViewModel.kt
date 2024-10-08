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
import net.mullvad.mullvadvpn.repository.WireguardConstraintsRepository
import net.mullvad.mullvadvpn.usecase.FilteredRelayListUseCase
import net.mullvad.mullvadvpn.usecase.SelectedLocationUseCase
import net.mullvad.mullvadvpn.usecase.customlists.CustomListsRelayItemUseCase
import net.mullvad.mullvadvpn.usecase.customlists.FilterCustomListsRelayItemUseCase

class SelectLocationListViewModel(
    private val relayListType: RelayListType,
    private val filteredRelayListUseCase: FilteredRelayListUseCase,
    private val filteredCustomListRelayItemsUseCase: FilterCustomListsRelayItemUseCase,
    private val selectedLocationUseCase: SelectedLocationUseCase,
    private val wireguardConstraintsRepository: WireguardConstraintsRepository,
    private val relayListRepository: RelayListRepository,
    customListsRelayItemUseCase: CustomListsRelayItemUseCase,
) : ViewModel() {
    private val _expandedItems: MutableStateFlow<Set<String>> =
        MutableStateFlow(initialExpand(initialSelection()))

    val uiState: StateFlow<SelectLocationListUiState> =
        combine(relayListItems(), customListsRelayItemUseCase()) { relayListItems, customLists ->
                SelectLocationListUiState.Content(
                    relayListItems = relayListItems,
                    customLists = customLists,
                )
            }
            .stateIn(viewModelScope, SharingStarted.Lazily, SelectLocationListUiState.Loading)

    fun onToggleExpand(item: RelayItemId, parent: CustomListId? = null, expand: Boolean) {
        _expandedItems.onToggleExpand(item, parent, expand)
    }

    private fun relayListItems() =
        combine(
            filteredRelayListUseCase(relayListType = relayListType),
            filteredCustomListRelayItemsUseCase(relayListType = relayListType),
            selectedLocationUseCase(),
            _expandedItems,
        ) { relayCountries, customLists, selectedItem, expandedItems ->
            relayListItems(
                relayCountries = relayCountries,
                relayListType = relayListType,
                customLists = customLists,
                selectedByThisEntryExitList =
                    selectedItem.selectedByThisEntryExitList(relayListType),
                selectedByOtherEntryExitList =
                    selectedItem.selectedByOtherEntryExitList(relayListType, customLists),
                expandedItems = expandedItems,
            )
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
