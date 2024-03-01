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
import net.mullvad.mullvadvpn.model.CreateCustomListResult
import net.mullvad.mullvadvpn.model.CustomListsError
import net.mullvad.mullvadvpn.model.UpdateCustomListResult
import net.mullvad.mullvadvpn.repository.CustomListsRepository

class CreateCustomListDialogViewModel(
    private val action: CustomListAction.Create,
    private val customListsRepository: CustomListsRepository,
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
            when (val result = customListsRepository.createCustomList(name)) {
                is CreateCustomListResult.Ok -> {
                    // We want to create the custom list with a location
                    if (action.locations.isNotEmpty()) {
                        addCustomListToLocation(
                            result.id,
                            name,
                            action.locations.first(),
                            action.locationNames.first()
                        )
                    } else {
                        // We want to create the custom list without a location
                        _uiSideEffect.send(
                            CreateCustomListDialogSideEffect.NavigateToCustomListLocationsScreen(
                                result.id
                            )
                        )
                    }
                }
                is CreateCustomListResult.Error -> {
                    _error.emit(result.error)
                }
            }
        }
    }

    private suspend fun addCustomListToLocation(
        customListId: String,
        name: String,
        locationCode: String,
        locationName: String
    ) {
        when (
            val result =
                customListsRepository.updateCustomListLocationsFromCodes(
                    customListId,
                    listOf(locationCode)
                )
        ) {
            is UpdateCustomListResult.Ok -> {
                _uiSideEffect.send(
                    CreateCustomListDialogSideEffect.ReturnWithResult(
                        CustomListResult.ListCreated(
                            locationName = locationName,
                            customListName = name,
                            reverseAction = action.not(customListId)
                        )
                    )
                )
            }
            is UpdateCustomListResult.Error -> {
                _error.emit(result.error)
            }
        }
    }

    fun clearError() {
        viewModelScope.launch { _error.emit(null) }
    }
}

sealed interface CreateCustomListDialogSideEffect {

    data class NavigateToCustomListLocationsScreen(val customListId: String) :
        CreateCustomListDialogSideEffect

    data class ReturnWithResult(val result: CustomListResult.ListCreated) :
        CreateCustomListDialogSideEffect
}
