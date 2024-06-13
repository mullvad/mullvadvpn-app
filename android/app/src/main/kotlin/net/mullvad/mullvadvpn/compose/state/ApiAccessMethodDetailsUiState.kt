package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.lib.model.ApiAccessMethodId
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethodName

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
        val isTestingAccessMethod: Boolean,
    ) : ApiAccessMethodDetailsUiState

    fun name() = (this as? Content)?.name?.value ?: ""

    fun canBeEdited() = this is Content && isEditable

    fun testingAccessMethod() = this is Content && isTestingAccessMethod

    fun currentMethod() = this is Content && isCurrentMethod
}
