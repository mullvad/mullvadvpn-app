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
import net.mullvad.mullvadvpn.compose.state.RelayFilterUiState
import net.mullvad.mullvadvpn.compose.state.toConstraintProviders
import net.mullvad.mullvadvpn.compose.state.toOwnershipConstraint
import net.mullvad.mullvadvpn.compose.state.toSelectedProviders
import net.mullvad.mullvadvpn.lib.model.Ownership
import net.mullvad.mullvadvpn.lib.model.ProviderId
import net.mullvad.mullvadvpn.repository.RelayListFilterRepository
import net.mullvad.mullvadvpn.usecase.ProviderToOwnershipsUseCase

class FilterViewModel(
    private val providerToOwnershipsUseCase: ProviderToOwnershipsUseCase,
    private val relayListFilterRepository: RelayListFilterRepository,
) : ViewModel() {
    private val _uiSideEffect = Channel<FilterScreenSideEffect>()
    val uiSideEffect = _uiSideEffect.receiveAsFlow()

    private val selectedOwnership = MutableStateFlow<Ownership?>(null)
    private val selectedProviders = MutableStateFlow<List<ProviderId>>(emptyList())

    init {
        viewModelScope.launch {
            selectedProviders.value =
                combine(
                        providerToOwnershipsUseCase(),
                        relayListFilterRepository.selectedProviders,
                    ) { providerToOwnerships, selectedConstraintProviders ->
                        selectedConstraintProviders.toSelectedProviders(
                            providerToOwnerships.keys.toList()
                        )
                    }
                    .first()

            val ownershipConstraint = relayListFilterRepository.selectedOwnership.first()
            selectedOwnership.value = ownershipConstraint.getOrNull()
        }
    }

    val uiState: StateFlow<RelayFilterUiState> =
        combine(providerToOwnershipsUseCase(), selectedOwnership, selectedProviders, ::createState)
            .stateIn(viewModelScope, SharingStarted.WhileSubscribed(), RelayFilterUiState())

    private fun createState(
        providerToOwnerships: Map<ProviderId, Set<Ownership>>,
        selectedOwnership: Ownership?,
        selectedProviders: List<ProviderId>,
    ): RelayFilterUiState =
        RelayFilterUiState(
            providerToOwnerships = providerToOwnerships,
            selectedOwnership = selectedOwnership,
            selectedProviders = selectedProviders,
        )

    fun setSelectedOwnership(ownership: Ownership?) {
        selectedOwnership.value = ownership
    }

    fun setSelectedProvider(checked: Boolean, provider: ProviderId) {
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
                    providerToOwnershipsUseCase().first().keys.toList()
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
                newSelectedProviders,
            )
            _uiSideEffect.send(FilterScreenSideEffect.CloseScreen)
        }
    }
}

sealed interface FilterScreenSideEffect {
    data object CloseScreen : FilterScreenSideEffect
}
