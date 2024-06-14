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
import net.mullvad.mullvadvpn.compose.communication.Created
import net.mullvad.mullvadvpn.compose.communication.CustomListAction
import net.mullvad.mullvadvpn.compose.state.CreateCustomListUiState
import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.CustomListName
import net.mullvad.mullvadvpn.lib.model.GeoLocationId
import net.mullvad.mullvadvpn.usecase.customlists.CreateWithLocationsError
import net.mullvad.mullvadvpn.usecase.customlists.CustomListActionUseCase

class CreateCustomListDialogViewModel(
    private val locationCode: GeoLocationId?,
    private val customListActionUseCase: CustomListActionUseCase,
) : ViewModel() {

    private val _uiSideEffect =
        Channel<CreateCustomListDialogSideEffect>(1, BufferOverflow.DROP_OLDEST)
    val uiSideEffect = _uiSideEffect.receiveAsFlow()

    private val _error = MutableStateFlow<CreateWithLocationsError?>(null)

    val uiState =
        _error
            .map { CreateCustomListUiState(it) }
            .stateIn(viewModelScope, SharingStarted.WhileSubscribed(), CreateCustomListUiState())

    fun createCustomList(name: String) {
        viewModelScope.launch {
            customListActionUseCase(
                    CustomListAction.Create(
                        CustomListName.fromString(name),
                        listOfNotNull(locationCode)
                    )
                )
                .fold(
                    { _error.emit(it) },
                    {
                        if (it.locationNames.isEmpty()) {
                            _uiSideEffect.send(
                                CreateCustomListDialogSideEffect
                                    .NavigateToCustomListLocationsScreen(it.id)
                            )
                        } else {
                            _uiSideEffect.send(
                                CreateCustomListDialogSideEffect.ReturnWithResult(it)
                            )
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

    data class NavigateToCustomListLocationsScreen(val customListId: CustomListId) :
        CreateCustomListDialogSideEffect

    data class ReturnWithResult(val result: Created) : CreateCustomListDialogSideEffect
}
