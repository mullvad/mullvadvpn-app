package net.mullvad.mullvadvpn.model

sealed interface GetAccountDataError {
    data class Unknown(val error: Throwable) : GetAccountDataError
}
