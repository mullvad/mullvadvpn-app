package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.lib.model.ApiAccessMethodSetting

data class ApiAccessListUiState(
    val currentApiAccessMethodSetting: ApiAccessMethodSetting? = null,
    val apiAccessMethodSettings: List<ApiAccessMethodSetting> = emptyList(),
)
