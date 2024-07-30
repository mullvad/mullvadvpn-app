package net.mullvad.mullvadvpn.lib.model

sealed interface GetVersionInfoError {
    data class Unknown(val error: Throwable) : GetVersionInfoError
}
