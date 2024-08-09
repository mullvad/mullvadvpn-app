package net.mullvad.mullvadvpn.lib.model

interface ClearAccountHistoryError {
    data class Unknown(val t: Throwable) : ClearAccountHistoryError
}
