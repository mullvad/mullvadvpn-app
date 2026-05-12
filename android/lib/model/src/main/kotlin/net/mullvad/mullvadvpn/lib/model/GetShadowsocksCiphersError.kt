package net.mullvad.mullvadvpn.lib.model

sealed interface GetShadowsocksCiphersError {
    data class Unknown(val throwable: Throwable) : GetShadowsocksCiphersError
}
