package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.lib.model.ApiAccessMethod
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethodId
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethodName
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethodSetting

sealed interface ApiAccessMethodDetailsUiState {
    val apiAccessMethodId: ApiAccessMethodId

    data class Loading(override val apiAccessMethodId: ApiAccessMethodId) :
        ApiAccessMethodDetailsUiState

    data class Content(
        val apiAccessMethodSetting: ApiAccessMethodSetting,
        val isDisableable: Boolean,
        val isCurrentMethod: Boolean,
        val isTestingAccessMethod: Boolean,
    ) : ApiAccessMethodDetailsUiState {
        override val apiAccessMethodId: ApiAccessMethodId = apiAccessMethodSetting.id
        val isEditable: Boolean =
            apiAccessMethodSetting.apiAccessMethod is ApiAccessMethod.CustomProxy
        val name: ApiAccessMethodName = apiAccessMethodSetting.name
        val enabled: Boolean = apiAccessMethodSetting.enabled
        val apiAccessMethod: ApiAccessMethod = apiAccessMethodSetting.apiAccessMethod
    }

    fun canBeEdited() = this is Content && apiAccessMethod is ApiAccessMethod.CustomProxy

    fun testingAccessMethod() = this is Content && isTestingAccessMethod

    fun currentMethod() = this is Content && isCurrentMethod
}
