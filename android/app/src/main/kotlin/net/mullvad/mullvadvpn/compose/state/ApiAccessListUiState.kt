package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.lib.model.ApiAccessMethod

data class ApiAccessListUiState(
    val currentApiAccessMethod: ApiAccessMethod? = null,
    val apiAccessMethods: List<ApiAccessMethod> = emptyList()
)
