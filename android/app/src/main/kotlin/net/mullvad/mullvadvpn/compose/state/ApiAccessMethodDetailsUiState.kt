package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.lib.model.ApiAccessMethodId
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethodName
import net.mullvad.mullvadvpn.usecase.TestApiAccessMethodState

sealed interface ApiAccessMethodDetailsUiState {
    val apiAccessMethodId: ApiAccessMethodId
    data class Loading(override val apiAccessMethodId: ApiAccessMethodId) : ApiAccessMethodDetailsUiState

    data class Content(
        override val apiAccessMethodId: ApiAccessMethodId,
        val name: ApiAccessMethodName,
        val enabled: Boolean,
        val canBeEdited: Boolean,
        val canBeDisabled: Boolean,
        val currentMethod: Boolean,
        val testApiAccessMethodState: TestApiAccessMethodState?
    ) : ApiAccessMethodDetailsUiState

    fun name() = (this as? Content)?.name?.value ?: ""

    fun canBeEdited() = (this as? Content)?.canBeEdited ?: false
}
