package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.compose.state.RelayFilterState
import net.mullvad.mullvadvpn.compose.state.toConstraintProviders
import net.mullvad.mullvadvpn.compose.state.toNullableOwnership
import net.mullvad.mullvadvpn.compose.state.toOwnershipConstraint
import net.mullvad.mullvadvpn.compose.state.toSelectedProviders
import net.mullvad.mullvadvpn.model.Ownership
import net.mullvad.mullvadvpn.relaylist.Provider
import net.mullvad.mullvadvpn.usecase.RelayListFilterUseCase

class FilterViewModel(
    private val relayListFilterUseCase: RelayListFilterUseCase,
) : ViewModel() {
    private val _uiSideEffect = MutableSharedFlow<FilterScreenSideEffect>()
    val uiSideEffect = _uiSideEffect.asSharedFlow()

    private val selectedOwnership = MutableStateFlow<Ownership?>(null)
    private val selectedProviders = MutableStateFlow<List<Provider>>(emptyList())

    init {
        viewModelScope.launch {
            selectedProviders.value =
                combine(
                        relayListFilterUseCase.availableProviders(),
                        relayListFilterUseCase.selectedProviders(),
                    ) { allProviders, selectedConstraintProviders ->
                        selectedConstraintProviders.toSelectedProviders(allProviders)
                    }
                    .first()

            val ownershipConstraint = relayListFilterUseCase.selectedOwnership().first()
            selectedOwnership.value = ownershipConstraint.toNullableOwnership()
        }
    }

    val uiState: StateFlow<RelayFilterState> =
        combine(
                selectedOwnership,
                relayListFilterUseCase.availableProviders(),
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
                    relayListFilterUseCase.availableProviders().first()
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
            relayListFilterUseCase.updateOwnershipAndProviderFilter(
                newSelectedOwnership,
                newSelectedProviders
            )
            _uiSideEffect.emit(FilterScreenSideEffect.CloseScreen)
        }
    }
}

sealed interface FilterScreenSideEffect {
    data object CloseScreen : FilterScreenSideEffect
}
