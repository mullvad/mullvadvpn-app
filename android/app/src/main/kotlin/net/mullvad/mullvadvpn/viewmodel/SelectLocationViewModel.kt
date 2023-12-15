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
import net.mullvad.mullvadvpn.compose.state.SelectLocationUiState
import net.mullvad.mullvadvpn.compose.state.toNullableOwnership
import net.mullvad.mullvadvpn.compose.state.toSelectedProviders
import net.mullvad.mullvadvpn.model.Constraint
import net.mullvad.mullvadvpn.model.Ownership
import net.mullvad.mullvadvpn.relaylist.Provider
import net.mullvad.mullvadvpn.relaylist.RelayItem
import net.mullvad.mullvadvpn.relaylist.filterOnSearchTerm
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManager
import net.mullvad.mullvadvpn.ui.serviceconnection.connectionProxy
import net.mullvad.mullvadvpn.usecase.RelayListFilterUseCase
import net.mullvad.mullvadvpn.usecase.RelayListUseCase

class SelectLocationViewModel(
    private val serviceConnectionManager: ServiceConnectionManager,
    private val relayListUseCase: RelayListUseCase,
    private val relayListFilterUseCase: RelayListFilterUseCase
) : ViewModel() {
    private val _searchTerm = MutableStateFlow(EMPTY_SEARCH_TERM)

    val uiState =
        combine(
                relayListUseCase.relayListWithSelection(),
                _searchTerm,
                relayListFilterUseCase.selectedOwnership(),
                relayListFilterUseCase.availableProviders(),
                relayListFilterUseCase.selectedProviders()
            ) {
                (relayCountries, relayItem),
                searchTerm,
                selectedOwnership,
                allProviders,
                selectedConstraintProviders ->
                val selectedProviders =
                    selectedConstraintProviders.toSelectedProviders(allProviders)

                val selectedProvidersByOwnershipList =
                    filterSelectedProvidersByOwnership(
                        selectedProviders,
                        selectedOwnership.toNullableOwnership()
                    )

                val allProvidersByOwnershipListList =
                    filterAllProvidersByOwnership(
                        allProviders,
                        selectedOwnership.toNullableOwnership()
                    )

                val filteredRelayCountries =
                    relayCountries.filterOnSearchTerm(searchTerm, relayItem)
                SelectLocationUiState.ShowData(
                    searchTerm = searchTerm,
                    countries = filteredRelayCountries,
                    selectedRelay = relayItem,
                    selectedOwnership = selectedOwnership.toNullableOwnership(),
                    selectedProvidersCount =
                        if (
                            selectedProvidersByOwnershipList.size ==
                                allProvidersByOwnershipListList.size
                        )
                            null
                        else selectedProvidersByOwnershipList.size
                )
            }
            .stateIn(
                viewModelScope,
                SharingStarted.WhileSubscribed(),
                SelectLocationUiState.Loading
            )

    private val _uiSideEffect = Channel<SelectLocationSideEffect>(1, BufferOverflow.DROP_OLDEST)
    val uiSideEffect = _uiSideEffect.receiveAsFlow()

    init {
        viewModelScope.launch { relayListUseCase.fetchRelayList() }
    }

    fun selectRelay(relayItem: RelayItem) {
        relayListUseCase.updateSelectedRelayLocation(relayItem.location)
        serviceConnectionManager.connectionProxy()?.connect()
        viewModelScope.launch { _uiSideEffect.send(SelectLocationSideEffect.CloseScreen) }
    }

    fun onSearchTermInput(searchTerm: String) {
        viewModelScope.launch { _searchTerm.emit(searchTerm) }
    }

    private fun filterSelectedProvidersByOwnership(
        selectedProviders: List<Provider>,
        selectedOwnership: Ownership?
    ): List<Provider> {
        return when (selectedOwnership) {
            Ownership.MullvadOwned -> selectedProviders.filter { it.mullvadOwned }
            Ownership.Rented -> selectedProviders.filterNot { it.mullvadOwned }
            else -> selectedProviders
        }
    }

    private fun filterAllProvidersByOwnership(
        allProviders: List<Provider>,
        selectedOwnership: Ownership?
    ): List<Provider> {
        return when (selectedOwnership) {
            Ownership.MullvadOwned -> allProviders.filter { it.mullvadOwned }
            Ownership.Rented -> allProviders.filterNot { it.mullvadOwned }
            else -> allProviders
        }
    }

    fun removeOwnerFilter() {
        viewModelScope.launch {
            relayListFilterUseCase.updateOwnershipAndProviderFilter(
                Constraint.Any(),
                relayListFilterUseCase.selectedProviders().first()
            )
        }
    }

    fun removeProviderFilter() {
        viewModelScope.launch {
            relayListFilterUseCase.updateOwnershipAndProviderFilter(
                relayListFilterUseCase.selectedOwnership().first(),
                Constraint.Any()
            )
        }
    }

    companion object {
        private const val EMPTY_SEARCH_TERM = ""
    }
}

sealed interface SelectLocationSideEffect {
    data object CloseScreen : SelectLocationSideEffect
}
