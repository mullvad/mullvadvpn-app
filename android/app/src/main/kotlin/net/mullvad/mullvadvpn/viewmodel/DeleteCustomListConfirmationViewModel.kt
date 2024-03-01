package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.compose.communication.CustomListAction
import net.mullvad.mullvadvpn.compose.communication.CustomListResult
import net.mullvad.mullvadvpn.model.GeographicLocationConstraint
import net.mullvad.mullvadvpn.repository.CustomListsRepository

class DeleteCustomListConfirmationViewModel(
    private val action: CustomListAction.Delete,
    private val customListsRepository: CustomListsRepository
) : ViewModel() {
    private val _uiSideEffect = Channel<DeleteCustomListConfirmationSideEffect>(Channel.BUFFERED)
    val uiSideEffect = _uiSideEffect.receiveAsFlow()

    fun deleteCustomList() {
        viewModelScope.launch {
            val oldLocations =
                customListsRepository.getCustomListById(action.customListId)?.locations?.map {
                    when (it) {
                        is GeographicLocationConstraint.City -> it.cityCode
                        is GeographicLocationConstraint.Country -> it.countryCode
                        is GeographicLocationConstraint.Hostname -> it.hostname
                    }
                } ?: emptyList()
            customListsRepository.deleteCustomList(id = action.customListId)
            _uiSideEffect.send(
                DeleteCustomListConfirmationSideEffect.ReturnWithResult(
                    result =
                        CustomListResult.ListDeleted(
                            name = action.name,
                            reverseAction = action.not(oldLocations)
                        )
                )
            )
        }
    }
}

sealed class DeleteCustomListConfirmationSideEffect {
    data class ReturnWithResult(val result: CustomListResult.ListDeleted) :
        DeleteCustomListConfirmationSideEffect()
}
