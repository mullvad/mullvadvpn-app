package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import net.mullvad.mullvadvpn.compose.state.CustomListsUiState
import net.mullvad.mullvadvpn.usecase.RelayListUseCase

class CustomListsViewModel(relayListUseCase: RelayListUseCase) : ViewModel() {
    val uiState =
        relayListUseCase
            .customLists()
            .map { CustomListsUiState.Content(it) }
            .stateIn(
                viewModelScope,
                started = SharingStarted.WhileSubscribed(),
                CustomListsUiState.Loading
            )
}
