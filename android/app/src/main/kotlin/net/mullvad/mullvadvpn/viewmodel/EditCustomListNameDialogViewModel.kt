package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.channels.BufferOverflow
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.usecase.CustomListUseCase

class EditCustomListNameDialogViewModel(
    private val id: String,
    private val customListUseCase: CustomListUseCase
) : ViewModel() {

    private val _uiSideEffect =
        Channel<EditCustomListNameDialogSideEffect>(1, BufferOverflow.DROP_OLDEST)
    val uiSideEffect = _uiSideEffect.receiveAsFlow()

    fun updateCustomListName(name: String) {
        viewModelScope.launch {
            if (customListUseCase.updateCustomListName(id, name)) {
                _uiSideEffect.send(EditCustomListNameDialogSideEffect.CloseScreen)
            } else {
                _uiSideEffect.send(EditCustomListNameDialogSideEffect.UpdateNameError)
            }
        }
    }
}

sealed interface EditCustomListNameDialogSideEffect {
    data object UpdateNameError : EditCustomListNameDialogSideEffect

    data object CloseScreen : EditCustomListNameDialogSideEffect
}
