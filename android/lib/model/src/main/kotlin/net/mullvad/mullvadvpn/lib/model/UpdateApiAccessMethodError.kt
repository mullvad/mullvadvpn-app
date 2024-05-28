package net.mullvad.mullvadvpn.lib.model

sealed interface UpdateApiAccessMethodError {
    data class Unknown(val t: Throwable) : UpdateApiAccessMethodError
}
