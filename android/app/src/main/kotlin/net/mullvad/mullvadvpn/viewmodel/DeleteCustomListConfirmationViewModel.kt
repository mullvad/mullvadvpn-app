package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.compose.communication.CustomListAction
import net.mullvad.mullvadvpn.compose.communication.CustomListResult
import net.mullvad.mullvadvpn.usecase.customlists.CustomListActionUseCase

class DeleteCustomListConfirmationViewModel(
    private val customListId: String,
    private val customListActionUseCase: CustomListActionUseCase
) : ViewModel() {
    private val _uiSideEffect = Channel<DeleteCustomListConfirmationSideEffect>(Channel.BUFFERED)
    val uiSideEffect = _uiSideEffect.receiveAsFlow()

    fun deleteCustomList() {
        viewModelScope.launch {
            val result =
                customListActionUseCase
                    .performAction(CustomListAction.Delete(customListId))
                    .getOrThrow()
            _uiSideEffect.send(DeleteCustomListConfirmationSideEffect.ReturnWithResult(result))
        }
    }
}

sealed class DeleteCustomListConfirmationSideEffect {
    data class ReturnWithResult(val result: CustomListResult.Deleted) :
        DeleteCustomListConfirmationSideEffect()
}
