package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.lib.model.ApiAccessMethodId
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethodName
import net.mullvad.mullvadvpn.lib.model.TestApiAccessMethodState

sealed interface ApiAccessMethodDetailsUiState {
    val apiAccessMethodId: ApiAccessMethodId

    data class Loading(override val apiAccessMethodId: ApiAccessMethodId) :
        ApiAccessMethodDetailsUiState

    data class Content(
        override val apiAccessMethodId: ApiAccessMethodId,
        val name: ApiAccessMethodName,
        val enabled: Boolean,
        val isEditable: Boolean,
        val isDisableable: Boolean,
        val isCurrentMethod: Boolean,
        val testApiAccessMethodState: TestApiAccessMethodState?
    ) : ApiAccessMethodDetailsUiState

    fun name() = (this as? Content)?.name?.value ?: ""

    fun canBeEdited() = (this as? Content)?.isEditable ?: false
}
