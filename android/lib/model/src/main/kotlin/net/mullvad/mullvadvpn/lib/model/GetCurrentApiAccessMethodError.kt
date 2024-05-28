package net.mullvad.mullvadvpn.lib.model

sealed interface GetCurrentApiAccessMethodError {
    data class Unknown(val t: Throwable) : GetCurrentApiAccessMethodError
}
