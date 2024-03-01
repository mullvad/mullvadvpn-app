package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.compose.communication.CustomListAction
import net.mullvad.mullvadvpn.compose.state.CustomListsUiState
import net.mullvad.mullvadvpn.model.CreateCustomListResult
import net.mullvad.mullvadvpn.repository.CustomListsRepository
import net.mullvad.mullvadvpn.usecase.RelayListUseCase

class CustomListsViewModel(
    relayListUseCase: RelayListUseCase,
    private val customListsRepository: CustomListsRepository
) : ViewModel() {

    val uiState =
        relayListUseCase
            .customLists()
            .map { CustomListsUiState.Content(it) }
            .stateIn(
                viewModelScope,
                started = SharingStarted.WhileSubscribed(),
                CustomListsUiState.Loading
            )

    fun undoDeleteCustomList(action: CustomListAction.Create) {
        viewModelScope.launch {
            val result = customListsRepository.createCustomList(action.name)
            if (result is CreateCustomListResult.Ok) {
                customListsRepository.updateCustomListLocationsFromCodes(
                    result.id,
                    action.locations
                )
            }
        }
    }
}
