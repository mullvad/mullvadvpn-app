package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.channels.BufferOverflow
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.compose.communication.CustomListAction
import net.mullvad.mullvadvpn.compose.state.SelectLocationUiState
import net.mullvad.mullvadvpn.compose.state.toNullableOwnership
import net.mullvad.mullvadvpn.compose.state.toSelectedProviders
import net.mullvad.mullvadvpn.model.Constraint
import net.mullvad.mullvadvpn.model.CreateCustomListResult
import net.mullvad.mullvadvpn.model.Ownership
import net.mullvad.mullvadvpn.relaylist.Provider
import net.mullvad.mullvadvpn.relaylist.RelayItem
import net.mullvad.mullvadvpn.relaylist.filterOnSearchTerm
import net.mullvad.mullvadvpn.relaylist.toLocationConstraint
import net.mullvad.mullvadvpn.repository.CustomListsRepository
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManager
import net.mullvad.mullvadvpn.ui.serviceconnection.connectionProxy
import net.mullvad.mullvadvpn.usecase.RelayListFilterUseCase
import net.mullvad.mullvadvpn.usecase.RelayListUseCase

class SelectLocationViewModel(
    private val serviceConnectionManager: ServiceConnectionManager,
    private val relayListUseCase: RelayListUseCase,
    private val relayListFilterUseCase: RelayListFilterUseCase,
    private val customListsRepository: CustomListsRepository
) : ViewModel() {
    private val _searchTerm = MutableStateFlow(EMPTY_SEARCH_TERM)

    val uiState =
        combine(
                relayListUseCase.relayListWithSelection(),
                _searchTerm,
                relayListFilterUseCase.selectedOwnership(),
                relayListFilterUseCase.availableProviders(),
                relayListFilterUseCase.selectedProviders(),
            ) {
                (customLists, relayCountries, selectedItem),
                searchTerm,
                selectedOwnership,
                allProviders,
                selectedConstraintProviders ->
                val selectedOwnershipItem = selectedOwnership.toNullableOwnership()
                val selectedProvidersCount =
                    filterSelectedProvidersByOwnership(
                            selectedConstraintProviders.toSelectedProviders(allProviders),
                            selectedOwnershipItem,
                        )
                        ?.size

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

    private val _uiSideEffect = Channel<SelectLocationSideEffect>(1, BufferOverflow.DROP_OLDEST)
    val uiSideEffect = _uiSideEffect.receiveAsFlow()

    init {
        viewModelScope.launch { relayListUseCase.fetchRelayList() }
    }

    fun selectRelay(relayItem: RelayItem) {
        val locationConstraint = relayItem.toLocationConstraint()
        relayListUseCase.updateSelectedRelayLocation(locationConstraint)
        serviceConnectionManager.connectionProxy()?.connect()
        viewModelScope.launch { _uiSideEffect.send(SelectLocationSideEffect.CloseScreen) }
    }

    fun onSearchTermInput(searchTerm: String) {
        viewModelScope.launch { _searchTerm.emit(searchTerm) }
    }

    private fun filterSelectedProvidersByOwnership(
        selectedProviders: List<Provider>?,
        selectedOwnership: Ownership?
    ): List<Provider>? =
        selectedProviders?.let {
            when (selectedOwnership) {
                Ownership.MullvadOwned -> selectedProviders.filter { it.mullvadOwned }
                Ownership.Rented -> selectedProviders.filterNot { it.mullvadOwned }
                else -> selectedProviders
            }
        }

    fun removeOwnerFilter() {
        viewModelScope.launch {
            relayListFilterUseCase.updateOwnershipAndProviderFilter(
                Constraint.Any(),
                relayListFilterUseCase.selectedProviders().first(),
            )
        }
    }

    fun removeProviderFilter() {
        viewModelScope.launch {
            relayListFilterUseCase.updateOwnershipAndProviderFilter(
                relayListFilterUseCase.selectedOwnership().first(),
                Constraint.Any(),
            )
        }
    }

    fun addLocationToList(item: RelayItem, customList: RelayItem.CustomList) {
        viewModelScope.launch {
            val newLocations = customList.locations + item
            customListsRepository.updateCustomListLocations(
                id = customList.id,
                locations = newLocations
            )
            _uiSideEffect.send(
                SelectLocationSideEffect.LocationAddedToCustomList(
                    item,
                    customList.copy(locations = newLocations)
                )
            )
        }
    }

    fun removeLocationFromList(item: RelayItem, customList: RelayItem.CustomList) {
        viewModelScope.launch {
            customListsRepository.updateCustomListLocations(
                customList.id,
                customList.locations - item
            )
        }
    }

    fun performAction(action: CustomListAction) {
        when (action) {
            is CustomListAction.Create -> createCustomList(action.name, action.locations)
            is CustomListAction.Delete -> deleteCustomList(action.customListId)
            is CustomListAction.Rename -> renameCustomList(action.customListId, action.name)
            is CustomListAction.UpdateLocations ->
                updateLocations(action.customListId, action.locations)
        }
    }

    private fun createCustomList(name: String, locations: List<String>) {
        viewModelScope.launch {
            val result = customListsRepository.createCustomList(name)
            if (result is CreateCustomListResult.Ok) {
                customListsRepository.updateCustomListLocationsFromCodes(result.id, locations)
            }
        }
    }

    private fun deleteCustomList(id: String) {
        customListsRepository.deleteCustomList(id)
    }

    private fun renameCustomList(id: String, name: String) {
        viewModelScope.launch { customListsRepository.updateCustomListName(id, name) }
    }

    private fun updateLocations(id: String, locations: List<String>) {
        viewModelScope.launch {
            customListsRepository.updateCustomListLocationsFromCodes(id, locations)
        }
    }

    companion object {
        private const val EMPTY_SEARCH_TERM = ""
    }
}

sealed interface SelectLocationSideEffect {
    data object CloseScreen : SelectLocationSideEffect

    data class LocationAddedToCustomList(
        val item: RelayItem,
        val customList: RelayItem.CustomList
    ) : SelectLocationSideEffect
}
