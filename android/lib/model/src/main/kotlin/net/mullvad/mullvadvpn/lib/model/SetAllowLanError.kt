package net.mullvad.mullvadvpn.lib.model

sealed interface SetAllowLanError {
    data class Unknown(val throwable: Throwable) : SetAllowLanError
}
