package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.repository.CustomListsRepository

class DeleteCustomListConfirmationViewModel(
    private val id: String,
    private val customListsRepository: CustomListsRepository
) : ViewModel() {
    private val _uiSideEffect = Channel<DeleteCustomListConfirmationSideEffect>(Channel.BUFFERED)
    val uiSideEffect = _uiSideEffect.receiveAsFlow()

    fun deleteCustomList() {
        viewModelScope.launch {
            customListsRepository.deleteCustomList(id = id)
            _uiSideEffect.send(DeleteCustomListConfirmationSideEffect.CloseDialog)
        }
    }
}

sealed class DeleteCustomListConfirmationSideEffect {
    data object CloseDialog : DeleteCustomListConfirmationSideEffect()
}
