package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.first
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
import net.mullvad.mullvadvpn.relaylist.Provider
import net.mullvad.mullvadvpn.relaylist.RelayItem
import net.mullvad.mullvadvpn.relaylist.filterOnSearchTerm
import net.mullvad.mullvadvpn.relaylist.toLocationConstraint
import net.mullvad.mullvadvpn.ui.serviceconnection.ConnectionProxy
import net.mullvad.mullvadvpn.usecase.RelayListFilterUseCase
import net.mullvad.mullvadvpn.usecase.RelayListUseCase
import net.mullvad.mullvadvpn.usecase.customlists.CustomListActionUseCase

class SelectLocationViewModel(
    private val relayListUseCase: RelayListUseCase,
    private val relayListFilterUseCase: RelayListFilterUseCase,
    private val customListActionUseCase: CustomListActionUseCase,
    private val connectionProxy: ConnectionProxy
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

    init {
        viewModelScope.launch { relayListUseCase.fetchRelayList() }
    }

    fun selectRelay(relayItem: RelayItem) {
        viewModelScope.launch {
            val locationConstraint = relayItem.toLocationConstraint()
            relayListUseCase.updateSelectedRelayLocation(locationConstraint)
            connectionProxy.connect()
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
        when (selectedOwnership) {
            Ownership.MullvadOwned -> selectedProviders.filter { it.mullvadOwned }
            Ownership.Rented -> selectedProviders.filterNot { it.mullvadOwned }
            else -> selectedProviders
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
            // If this is null then something is seriously wrong
            val newLocation = item.location()!!
            val newLocations =
                (customList.locations.map { it.location() } + newLocation).filterNotNull()
            customListActionUseCase
                .performAction(CustomListAction.UpdateLocations(customList.id, newLocations))
                .fold(
                    { TODO("We should probably handle this error") },
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
}
