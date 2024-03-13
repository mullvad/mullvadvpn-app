package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.channels.BufferOverflow
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.compose.communication.CustomListAction
import net.mullvad.mullvadvpn.compose.communication.CustomListResult
import net.mullvad.mullvadvpn.compose.state.CreateCustomListUiState
import net.mullvad.mullvadvpn.model.CustomListsError
import net.mullvad.mullvadvpn.usecase.customlists.CustomListActionUseCase
import net.mullvad.mullvadvpn.usecase.customlists.CustomListsException

class CreateCustomListDialogViewModel(
    private val locationCode: String,
    private val customListActionUseCase: CustomListActionUseCase,
) : ViewModel() {

    private val _uiSideEffect =
        Channel<CreateCustomListDialogSideEffect>(1, BufferOverflow.DROP_OLDEST)
    val uiSideEffect = _uiSideEffect.receiveAsFlow()

    private val _error = MutableStateFlow<CustomListsError?>(null)

    val uiState =
        _error
            .map { CreateCustomListUiState(it) }
            .stateIn(viewModelScope, SharingStarted.WhileSubscribed(), CreateCustomListUiState())

    fun createCustomList(name: String) {
        viewModelScope.launch {
            customListActionUseCase
                .performAction(
                    CustomListAction.Create(
                        name,
                        if (locationCode.isNotEmpty()) {
                            listOf(locationCode)
                        } else {
                            emptyList()
                        }
                    )
                )
                .fold(
                    onSuccess = { result ->
                        if (result.locationName != null) {
                            _uiSideEffect.send(
                                CreateCustomListDialogSideEffect.ReturnWithResult(result)
                            )
                        } else {
                            _uiSideEffect.send(
                                CreateCustomListDialogSideEffect
                                    .NavigateToCustomListLocationsScreen(result.id)
                            )
                        }
                    },
                    onFailure = { error ->
                        if (error is CustomListsException) {
                            _error.emit(error.error)
                        } else {
                            _error.emit(CustomListsError.OtherError)
                        }
                    }
                )
        }
    }

    fun clearError() {
        viewModelScope.launch { _error.emit(null) }
    }
}

sealed interface CreateCustomListDialogSideEffect {

    data class NavigateToCustomListLocationsScreen(val customListId: String) :
        CreateCustomListDialogSideEffect

    data class ReturnWithResult(val result: CustomListResult.Created) :
        CreateCustomListDialogSideEffect
}
