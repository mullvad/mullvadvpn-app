package net.mullvad.mullvadvpn.lib.model

data class ApiAccessMethodSetting(
    val id: ApiAccessMethodId,
    val name: ApiAccessMethodName,
    val enabled: Boolean,
    val apiAccessMethod: ApiAccessMethod,
)
