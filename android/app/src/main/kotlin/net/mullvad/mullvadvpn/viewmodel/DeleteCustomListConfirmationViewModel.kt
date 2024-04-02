package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.compose.communication.CustomListAction
import net.mullvad.mullvadvpn.compose.communication.CustomListResult
import net.mullvad.mullvadvpn.model.CustomListId
import net.mullvad.mullvadvpn.usecase.customlists.CustomListActionUseCase

class DeleteCustomListConfirmationViewModel(
    private val customListId: CustomListId,
    private val customListActionUseCase: CustomListActionUseCase
) : ViewModel() {
    private val _uiSideEffect = Channel<DeleteCustomListConfirmationSideEffect>(Channel.BUFFERED)
    val uiSideEffect = _uiSideEffect.receiveAsFlow()

    fun deleteCustomList() {
        viewModelScope.launch {
            customListActionUseCase
                .performAction(CustomListAction.Delete(customListId))
                .fold(
                    { TODO("We should totally handle this") },
                    {
                        _uiSideEffect.send(
                            DeleteCustomListConfirmationSideEffect.ReturnWithResult(it)
                        )
                    }
                )
        }
    }
}

sealed class DeleteCustomListConfirmationSideEffect {
    data class ReturnWithResult(val result: CustomListResult.Deleted) :
        DeleteCustomListConfirmationSideEffect()
}
