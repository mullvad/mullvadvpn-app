package net.mullvad.mullvadvpn.lib.model

sealed interface RemoveApiAccessMethodError {
    data class Unknown(val t: Throwable) : RemoveApiAccessMethodError
}
