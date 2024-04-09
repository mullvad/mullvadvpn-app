package net.mullvad.mullvadvpn.model

sealed interface SetDnsOptionsError {
    data class Unknown(val throwable: Throwable) : SetDnsOptionsError
}
