package net.mullvad.mullvadvpn.lib.model

sealed interface GetAccountHistoryError {
    data class Unknown(val error: Throwable) :
        net.mullvad.mullvadvpn.lib.model.GetAccountHistoryError
}
