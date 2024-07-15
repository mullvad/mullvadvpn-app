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
import net.mullvad.mullvadvpn.compose.communication.LocationsChanged
import net.mullvad.mullvadvpn.compose.state.SelectLocationUiState
import net.mullvad.mullvadvpn.compose.state.toNullableOwnership
import net.mullvad.mullvadvpn.compose.state.toSelectedProviders
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.Ownership
import net.mullvad.mullvadvpn.lib.model.Provider
import net.mullvad.mullvadvpn.lib.model.Providers
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.relaylist.filterOnOwnershipAndProvider
import net.mullvad.mullvadvpn.relaylist.filterOnSearchTerm
import net.mullvad.mullvadvpn.repository.RelayListFilterRepository
import net.mullvad.mullvadvpn.repository.RelayListRepository
import net.mullvad.mullvadvpn.usecase.AvailableProvidersUseCase
import net.mullvad.mullvadvpn.usecase.FilteredRelayListUseCase
import net.mullvad.mullvadvpn.usecase.customlists.CustomListActionUseCase
import net.mullvad.mullvadvpn.usecase.customlists.CustomListsRelayItemUseCase
import net.mullvad.mullvadvpn.util.combine

class SelectLocationViewModel(
    private val relayListFilterRepository: RelayListFilterRepository,
    availableProvidersUseCase: AvailableProvidersUseCase,
    customListsRelayItemUseCase: CustomListsRelayItemUseCase,
    private val customListActionUseCase: CustomListActionUseCase,
    filteredRelayListUseCase: FilteredRelayListUseCase,
    private val relayListRepository: RelayListRepository
) : ViewModel() {
    private val _searchTerm = MutableStateFlow(EMPTY_SEARCH_TERM)

    @Suppress("DestructuringDeclarationWithTooManyEntries")
    val uiState =
        combine(
                filteredRelayListUseCase(),
                customListsRelayItemUseCase(),
                relayListRepository.selectedLocation,
                _searchTerm,
                relayListFilterRepository.selectedOwnership,
                availableProvidersUseCase(),
                relayListFilterRepository.selectedProviders,
            ) {
                relayCountries,
                customLists,
                selectedItem,
                searchTerm,
                selectedOwnership,
                allProviders,
                selectedConstraintProviders ->
                val selectRelayItemId = selectedItem.getOrNull()
                val selectedOwnershipItem = selectedOwnership.toNullableOwnership()
                val selectedProvidersCount =
                    when (selectedConstraintProviders) {
                        is Constraint.Any -> null
                        is Constraint.Only ->
                            filterSelectedProvidersByOwnership(
                                    selectedConstraintProviders.toSelectedProviders(allProviders),
                                    selectedOwnershipItem,
                                )
                                .size
                    }

                val filteredRelayCountries =
                    relayCountries.filterOnSearchTerm(searchTerm, selectRelayItemId)

                val filteredCustomLists =
                    customLists
                        .filterOnSearchTerm(searchTerm)
                        .filterOnOwnershipAndProvider(
                            ownership = selectedOwnership,
                            providers = selectedConstraintProviders,
                        )

                SelectLocationUiState.Content(
                    searchTerm = searchTerm,
                    selectedOwnership = selectedOwnershipItem,
                    selectedProvidersCount = selectedProvidersCount,
                    filteredCustomLists = filteredCustomLists,
                    customLists = customLists,
                    countries = filteredRelayCountries,
                    selectedItem = selectRelayItemId,
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
            val locationConstraint = relayItem.id
            relayListRepository
                .updateSelectedRelayLocation(locationConstraint)
                .fold(
                    { _uiSideEffect.trySend(SelectLocationSideEffect.GenericError) },
                    { _uiSideEffect.trySend(SelectLocationSideEffect.CloseScreen) },
                )
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
        viewModelScope.launch { relayListFilterRepository.updateSelectedOwnership(Constraint.Any) }
    }

    fun removeProviderFilter() {
        viewModelScope.launch { relayListFilterRepository.updateSelectedProviders(Constraint.Any) }
    }

    fun performAction(action: CustomListAction) {
        viewModelScope.launch { customListActionUseCase(action) }
    }

    private fun List<RelayItem.CustomList>.filterOnOwnershipAndProvider(
        ownership: Constraint<Ownership>,
        providers: Constraint<Providers>
    ): List<RelayItem.CustomList> = map { item ->
        item.filterOnOwnershipAndProvider(ownership, providers)
    }

    companion object {
        private const val EMPTY_SEARCH_TERM = ""
    }
}

sealed interface SelectLocationSideEffect {
    data object CloseScreen : SelectLocationSideEffect

    data class LocationAddedToCustomList(val result: LocationsChanged) : SelectLocationSideEffect

    class LocationRemovedFromCustomList(val result: LocationsChanged) : SelectLocationSideEffect

    data object GenericError : SelectLocationSideEffect
}
