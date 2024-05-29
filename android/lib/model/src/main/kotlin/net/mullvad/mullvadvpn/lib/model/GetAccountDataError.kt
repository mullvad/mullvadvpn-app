package net.mullvad.mullvadvpn.lib.model

sealed interface GetAccountDataError {
    data class Unknown(val error: Throwable) : GetAccountDataError
}
