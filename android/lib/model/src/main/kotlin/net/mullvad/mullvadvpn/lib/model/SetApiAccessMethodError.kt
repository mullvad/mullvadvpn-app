package net.mullvad.mullvadvpn.lib.model

sealed interface SetApiAccessMethodError {
    data class Unknown(val t: Throwable) : SetApiAccessMethodError
}
