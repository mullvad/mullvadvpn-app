package net.mullvad.mullvadvpn.lib.model

sealed interface SetDnsOptionsError {
    data class Unknown(val throwable: Throwable) : SetDnsOptionsError
}
