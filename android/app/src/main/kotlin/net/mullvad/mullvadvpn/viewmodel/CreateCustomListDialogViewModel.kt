package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.channels.BufferOverflow
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.repository.CustomListsRepository

class CreateCustomListDialogViewModel(
    private val customListsRepository: CustomListsRepository,
) : ViewModel() {

    private val _uiSideEffect =
        Channel<CreateCustomListDialogSideEffect>(1, BufferOverflow.DROP_OLDEST)
    val uiSideEffect = _uiSideEffect.receiveAsFlow()

    fun createCustomList(name: String) {
        viewModelScope.launch {
            customListsRepository.createCustomList(name)?.let { id ->
                _uiSideEffect.send(CreateCustomListDialogSideEffect.NavigateToCustomListScreen(id))
            } ?: _uiSideEffect.send(CreateCustomListDialogSideEffect.CreateCustomListError)
        }
    }
}

sealed interface CreateCustomListDialogSideEffect {
    data object CreateCustomListError : CreateCustomListDialogSideEffect

    data class NavigateToCustomListScreen(val customListId: String) :
        CreateCustomListDialogSideEffect
}
