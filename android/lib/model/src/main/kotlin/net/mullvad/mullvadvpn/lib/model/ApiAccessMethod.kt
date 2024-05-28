package net.mullvad.mullvadvpn.lib.model

data class ApiAccessMethod(
    val id: ApiAccessMethodId,
    val name: ApiAccessMethodName,
    val enabled: Boolean,
    val apiAccessMethodType: ApiAccessMethodType
)
