package net.mullvad.mullvadvpn.lib.model

sealed interface GetAccountHistoryError {
    data class Unknown(val error: Throwable) : GetAccountHistoryError
}
