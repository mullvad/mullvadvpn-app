package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlin.collections.plus
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.WhileSubscribed
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.compose.state.RelayFilterUiState
import net.mullvad.mullvadvpn.constant.VIEW_MODEL_STOP_TIMEOUT
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.Ownership
import net.mullvad.mullvadvpn.lib.model.ProviderId
import net.mullvad.mullvadvpn.lib.model.Providers
import net.mullvad.mullvadvpn.repository.RelayListFilterRepository
import net.mullvad.mullvadvpn.usecase.ProviderToOwnershipsUseCase

class FilterViewModel(
    providerToOwnershipsUseCase: ProviderToOwnershipsUseCase,
    private val relayListFilterRepository: RelayListFilterRepository,
) : ViewModel() {
    private val _uiSideEffect = Channel<FilterScreenSideEffect>()
    val uiSideEffect = _uiSideEffect.receiveAsFlow()

    private val selectedOwnership = MutableStateFlow<Constraint<Ownership>>(Constraint.Any)
    private val selectedProviders = MutableStateFlow<Constraint<Providers>>(Constraint.Any)

    init {
        viewModelScope.launch {
            selectedProviders.value = relayListFilterRepository.selectedProviders.first()
            selectedOwnership.value = relayListFilterRepository.selectedOwnership.first()
        }
    }

    val uiState: StateFlow<RelayFilterUiState> =
        combine(providerToOwnershipsUseCase(), selectedOwnership, selectedProviders, ::createState)
            .stateIn(
                viewModelScope,
                SharingStarted.WhileSubscribed(VIEW_MODEL_STOP_TIMEOUT),
                RelayFilterUiState(),
            )

    private fun createState(
        providerToOwnerships: Map<ProviderId, Set<Ownership>>,
        selectedOwnership: Constraint<Ownership>,
        selectedProviders: Constraint<Providers>,
    ): RelayFilterUiState =
        RelayFilterUiState(
            providerToOwnerships = providerToOwnerships,
            selectedOwnership = selectedOwnership,
            selectedProviders = selectedProviders,
        )

    fun setSelectedOwnership(ownership: Constraint<Ownership>) {
        selectedOwnership.value = ownership
    }

    fun setSelectedProvider(checked: Boolean, provider: ProviderId) {
        selectedProviders.update {
            if (checked) {
                it.check(provider, uiState.value.allProviders)
            } else {
                it.uncheck(provider, uiState.value.allProviders)
            }
        }
    }

    private fun Constraint<Providers>.check(
        provider: ProviderId,
        allProviders: Providers,
    ): Constraint<Providers> {
        return when (this) {
            is Constraint.Any -> Constraint.Any
            is Constraint.Only -> {
                val newProviderList = value + provider
                if (allProviders.size == newProviderList.size) {
                    Constraint.Any
                } else {
                    Constraint.Only(newProviderList)
                }
            }
        }
    }

    private fun Constraint<Providers>.uncheck(
        provider: ProviderId,
        allProviders: Providers,
    ): Constraint<Providers> {
        return when (this) {
            is Constraint.Any -> Constraint.Only(allProviders - provider)
            is Constraint.Only -> Constraint.Only(value - provider)
        }
    }

    fun setAllProviders(isChecked: Boolean) {
        viewModelScope.launch {
            selectedProviders.value =
                if (isChecked) {
                    Constraint.Any
                } else {
                    Constraint.Only(emptySet())
                }
        }
    }

    fun onApplyButtonClicked() {
        val newSelectedOwnership = selectedOwnership.value
        val newSelectedProviders = selectedProviders.value

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
