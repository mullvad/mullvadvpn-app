package net.mullvad.mullvadvpn.lib.model

sealed interface SetDaitaError {
    data class Unknown(val throwable: Throwable) : SetDaitaError
}
