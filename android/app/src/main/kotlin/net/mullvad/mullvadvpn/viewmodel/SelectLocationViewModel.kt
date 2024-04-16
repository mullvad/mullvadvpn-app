package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.compose.communication.CustomListAction
import net.mullvad.mullvadvpn.compose.communication.CustomListResult
import net.mullvad.mullvadvpn.compose.state.SelectLocationUiState
import net.mullvad.mullvadvpn.compose.state.toNullableOwnership
import net.mullvad.mullvadvpn.compose.state.toSelectedProviders
import net.mullvad.mullvadvpn.model.Constraint
import net.mullvad.mullvadvpn.model.Ownership
import net.mullvad.mullvadvpn.model.Provider
import net.mullvad.mullvadvpn.model.RelayItem
import net.mullvad.mullvadvpn.relaylist.filterOnSearchTerm
import net.mullvad.mullvadvpn.relaylist.toLocationConstraint
import net.mullvad.mullvadvpn.repository.RelayListFilterRepository
import net.mullvad.mullvadvpn.repository.SelectedLocationRepository
import net.mullvad.mullvadvpn.ui.serviceconnection.ConnectionProxy
import net.mullvad.mullvadvpn.usecase.AvailableProvidersUseCase
import net.mullvad.mullvadvpn.usecase.FilteredRelayListUseCase
import net.mullvad.mullvadvpn.usecase.SelectedLocationRelayItemUseCase
import net.mullvad.mullvadvpn.usecase.customlists.CustomListActionUseCase
import net.mullvad.mullvadvpn.usecase.customlists.CustomListsRelayItemUseCase
import net.mullvad.mullvadvpn.util.combine

class SelectLocationViewModel(
    private val relayListFilterRepository: RelayListFilterRepository,
    availableProvidersUseCase: AvailableProvidersUseCase,
    customListsRelayItemUseCase: CustomListsRelayItemUseCase,
    selectedLocationRelayItemUseCase: SelectedLocationRelayItemUseCase,
    private val customListActionUseCase: CustomListActionUseCase,
    filteredRelayListUseCase: FilteredRelayListUseCase,
    private val selectedLocationRepository: SelectedLocationRepository
) : ViewModel() {
    private val _searchTerm = MutableStateFlow(EMPTY_SEARCH_TERM)

    val uiState =
        combine(
                filteredRelayListUseCase.filteredRelayList(),
                customListsRelayItemUseCase.customListsRelayItems(),
                selectedLocationRelayItemUseCase.selectedRelayItem(),
                _searchTerm,
                relayListFilterRepository.selectedOwnership,
                availableProvidersUseCase.availableProviders(),
                relayListFilterRepository.selectedProviders,
            ) {
                relayCountries,
                customLists,
                selectedItem,
                searchTerm,
                selectedOwnership,
                allProviders,
                selectedConstraintProviders ->
                val selectedOwnershipItem = selectedOwnership.toNullableOwnership()
                val selectedProvidersCount =
                    when (selectedConstraintProviders) {
                        is Constraint.Any -> null
                        is Constraint.Only ->
                            filterSelectedProvidersByOwnership(
                                    selectedConstraintProviders.toSelectedProviders(allProviders),
                                    selectedOwnershipItem
                                )
                                .size
                    }

                val filteredRelayCountries =
                    relayCountries.filterOnSearchTerm(searchTerm, selectedItem)

                val filteredCustomLists = customLists.filterOnSearchTerm(searchTerm)

                SelectLocationUiState.Content(
                    searchTerm = searchTerm,
                    selectedOwnership = selectedOwnershipItem,
                    selectedProvidersCount = selectedProvidersCount,
                    filteredCustomLists = filteredCustomLists,
                    customLists = customLists,
                    countries = filteredRelayCountries,
                    selectedItem = selectedItem,
                )
            }
            .stateIn(
                viewModelScope,
                SharingStarted.WhileSubscribed(),
                SelectLocationUiState.Loading,
            )

    private val _uiSideEffect = Channel<SelectLocationSideEffect>()
    val uiSideEffect = _uiSideEffect.receiveAsFlow()

    fun selectRelay(relayItem: RelayItem) {
        viewModelScope.launch {
            val locationConstraint = relayItem.toLocationConstraint()
            selectedLocationRepository.updateSelectedRelayLocation(locationConstraint)
            _uiSideEffect.trySend(SelectLocationSideEffect.CloseScreen)
        }
    }

    fun onSearchTermInput(searchTerm: String) {
        viewModelScope.launch { _searchTerm.emit(searchTerm) }
    }

    private fun filterSelectedProvidersByOwnership(
        selectedProviders: List<Provider>,
        selectedOwnership: Ownership?
    ): List<Provider> =
        if (selectedOwnership == null) selectedProviders
        else selectedProviders.filter { it.ownership == selectedOwnership }

    fun removeOwnerFilter() {
        viewModelScope.launch {
            relayListFilterRepository.updateSelectedOwnership(Constraint.Any)
        }
    }

    fun removeProviderFilter() {
        viewModelScope.launch {
            relayListFilterRepository.updateSelectedProviders(Constraint.Any)
        }
    }

    fun addLocationToList(item: RelayItem.Location, customList: RelayItem.CustomList) {
        viewModelScope.launch {
            // If this is null then something is seriously wrong
            val newLocation = item.location
            val newLocations = (customList.locations.map { it.location } + newLocation)
            customListActionUseCase
                .performAction(CustomListAction.UpdateLocations(customList.id, newLocations))
                .fold(
                    { _uiSideEffect.send(SelectLocationSideEffect.GenericError) },
                    { _uiSideEffect.send(SelectLocationSideEffect.LocationAddedToCustomList(it)) }
                )
        }
    }

    fun performAction(action: CustomListAction) {
        viewModelScope.launch { customListActionUseCase.performAction(action) }
    }

    companion object {
        private const val EMPTY_SEARCH_TERM = ""
    }
}

sealed interface SelectLocationSideEffect {
    data object CloseScreen : SelectLocationSideEffect

    data class LocationAddedToCustomList(val result: CustomListResult.LocationsChanged) :
        SelectLocationSideEffect

    data object GenericError : SelectLocationSideEffect
}
