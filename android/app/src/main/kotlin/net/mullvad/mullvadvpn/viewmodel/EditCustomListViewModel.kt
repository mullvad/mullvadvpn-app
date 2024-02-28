package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.channels.BufferOverflow
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.flow.stateIn
import net.mullvad.mullvadvpn.compose.state.EditCustomListState
import net.mullvad.mullvadvpn.repository.CustomListsRepository
import net.mullvad.mullvadvpn.usecase.RelayListUseCase

class EditCustomListViewModel(
    private val customListId: String,
    relayListUseCase: RelayListUseCase,
    private val customListsRepository: CustomListsRepository
) : ViewModel() {

    private val _uiSideEffect = Channel<EditCustomListSideEffect>(1, BufferOverflow.DROP_OLDEST)
    val uiSideEffect = _uiSideEffect.receiveAsFlow()

    val uiState =
        relayListUseCase
            .customLists()
            .map { customLists ->
                customLists
                    .find { it.id == customListId }
                    .let {
                        EditCustomListState.Content(
                            id = it?.id ?: "",
                            name = it?.name ?: "",
                            numberOfLocations = it?.locations?.size ?: 0
                        )
                    }
            }
            .stateIn(viewModelScope, SharingStarted.WhileSubscribed(), EditCustomListState.Loading)
}

sealed interface EditCustomListSideEffect {
    data object CloseScreen : EditCustomListSideEffect
}
