package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.lib.model.ApiAccessMethodName
import net.mullvad.mullvadvpn.usecase.TestApiAccessMethodState

sealed interface ApiAccessMethodDetailsUiState {
    data object Loading : ApiAccessMethodDetailsUiState

    data class Content(
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
