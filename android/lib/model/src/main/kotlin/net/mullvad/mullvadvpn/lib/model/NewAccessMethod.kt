package net.mullvad.mullvadvpn.lib.model

data class NewAccessMethod(
    val name: ApiAccessMethodName,
    val enabled: Boolean,
    val apiAccessMethodType: ApiAccessMethodType
)
