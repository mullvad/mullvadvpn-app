package net.mullvad.mullvadvpn.model

sealed interface GetAccountHistoryError {
    data class Unknown(val error: Throwable) : GetAccountHistoryError
}
