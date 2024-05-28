package net.mullvad.mullvadvpn.lib.model

sealed interface AddApiAccessMethodError {
    data class Unknown(val t: Throwable) : AddApiAccessMethodError
}
