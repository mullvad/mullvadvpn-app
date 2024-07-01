package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.ramcosta.composedestinations.generated.destinations.EditCustomListDestination
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import net.mullvad.mullvadvpn.compose.state.EditCustomListState
import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.repository.CustomListsRepository

class EditCustomListViewModel(
    customListsRepository: CustomListsRepository,
    savedStateHandle: SavedStateHandle
) : ViewModel() {
    private val customListId: CustomListId =
        EditCustomListDestination.argsFrom(savedStateHandle).customListId

    val uiState =
        customListsRepository.customLists
            .map { customLists ->
                customLists
                    ?.find { it.id == customListId }
                    ?.let {
                        EditCustomListState.Content(
                            id = it.id,
                            name = it.name,
                            locations = it.locations
                        )
                    } ?: EditCustomListState.NotFound
            }
            .stateIn(viewModelScope, SharingStarted.WhileSubscribed(), EditCustomListState.Loading)
}
