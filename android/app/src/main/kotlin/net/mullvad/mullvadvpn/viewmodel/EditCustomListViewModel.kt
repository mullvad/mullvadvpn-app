package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import net.mullvad.mullvadvpn.compose.state.EditCustomListState
import net.mullvad.mullvadvpn.usecase.RelayListUseCase

class EditCustomListViewModel(
    private val customListId: String,
    relayListUseCase: RelayListUseCase
) : ViewModel() {
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
                            locations = it?.locations ?: emptyList()
                        )
                    }
            }
            .stateIn(viewModelScope, SharingStarted.WhileSubscribed(), EditCustomListState.Loading)
}
