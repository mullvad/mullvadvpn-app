package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.compose.state.RelayFilterState
import net.mullvad.mullvadvpn.compose.state.toConstraintProviders
import net.mullvad.mullvadvpn.compose.state.toNullableOwnership
import net.mullvad.mullvadvpn.compose.state.toOwnershipConstraint
import net.mullvad.mullvadvpn.compose.state.toSelectedProviders
import net.mullvad.mullvadvpn.lib.model.Ownership
import net.mullvad.mullvadvpn.lib.model.Provider
import net.mullvad.mullvadvpn.repository.RelayListFilterRepository
import net.mullvad.mullvadvpn.usecase.AvailableProvidersUseCase

class FilterViewModel(
    private val availableProvidersUseCase: AvailableProvidersUseCase,
    private val relayListFilterRepository: RelayListFilterRepository
) : ViewModel() {
    private val _uiSideEffect = Channel<FilterScreenSideEffect>()
    val uiSideEffect = _uiSideEffect.receiveAsFlow()

    private val selectedOwnership = MutableStateFlow<Ownership?>(null)
    private val selectedProviders = MutableStateFlow<List<Provider>>(emptyList())

    init {
        viewModelScope.launch {
            selectedProviders.value =
                combine(
                        availableProvidersUseCase(),
                        relayListFilterRepository.selectedProviders,
                    ) { allProviders, selectedConstraintProviders ->
                        selectedConstraintProviders.toSelectedProviders(allProviders)
                    }
                    .first()

            val ownershipConstraint = relayListFilterRepository.selectedOwnership.first()
            selectedOwnership.value = ownershipConstraint.toNullableOwnership()
        }
    }

    val uiState: StateFlow<RelayFilterState> =
        combine(
                selectedOwnership,
                availableProvidersUseCase(),
                selectedProviders,
            ) { selectedOwnership, allProviders, selectedProviders ->
                RelayFilterState(
                    selectedOwnership = selectedOwnership,
                    allProviders = allProviders,
                    selectedProviders = selectedProviders
                )
            }
            .stateIn(
                viewModelScope,
                SharingStarted.WhileSubscribed(),
                RelayFilterState(
                    allProviders = emptyList(),
                    selectedOwnership = null,
                    selectedProviders = emptyList()
                ),
            )

    fun setSelectedOwnership(ownership: Ownership?) {
        selectedOwnership.value = ownership
    }

    fun setSelectedProvider(checked: Boolean, provider: Provider) {
        selectedProviders.value =
            if (checked) {
                selectedProviders.value + provider
            } else {
                selectedProviders.value - provider
            }
    }

    fun setAllProviders(isChecked: Boolean) {
        viewModelScope.launch {
            selectedProviders.value =
                if (isChecked) {
                    availableProvidersUseCase().first()
                } else {
                    emptyList()
                }
        }
    }

    fun onApplyButtonClicked() {
        val newSelectedOwnership = selectedOwnership.value.toOwnershipConstraint()
        val newSelectedProviders =
            selectedProviders.value.toConstraintProviders(uiState.value.allProviders)

        viewModelScope.launch {
            relayListFilterRepository.updateSelectedOwnershipAndProviderFilter(
                newSelectedOwnership,
                newSelectedProviders
            )
            _uiSideEffect.send(FilterScreenSideEffect.CloseScreen)
        }
    }
}

sealed interface FilterScreenSideEffect {
    data object CloseScreen : FilterScreenSideEffect
}
