package net.mullvad.mullvadvpn.apiaccess.impl.screen.list

import net.mullvad.mullvadvpn.lib.model.ApiAccessMethodSetting

data class ApiAccessListUiState(
    val currentApiAccessMethodSetting: ApiAccessMethodSetting? = null,
    val apiAccessMethodSettings: List<ApiAccessMethodSetting> = emptyList(),
)
