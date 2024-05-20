package net.mullvad.mullvadvpn.model

sealed interface SetAllowLanError {
    data class Unknown(val throwable: Throwable) : SetAllowLanError
}
