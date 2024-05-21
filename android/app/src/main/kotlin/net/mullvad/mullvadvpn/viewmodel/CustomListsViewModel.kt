package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.compose.communication.CustomListAction
import net.mullvad.mullvadvpn.compose.state.CustomListsUiState
import net.mullvad.mullvadvpn.repository.CustomListsRepository
import net.mullvad.mullvadvpn.usecase.customlists.CustomListActionUseCase

class CustomListsViewModel(
    customListsRepository: CustomListsRepository,
    private val customListActionUseCase: CustomListActionUseCase
) : ViewModel() {

    val uiState =
        customListsRepository.customLists
            .filterNotNull()
            .map(CustomListsUiState::Content)
            .stateIn(
                viewModelScope,
                started = SharingStarted.WhileSubscribed(),
                CustomListsUiState.Loading
            )

    fun undoDeleteCustomList(action: CustomListAction.Create) {
        viewModelScope.launch { customListActionUseCase.performAction(action) }
    }
}
