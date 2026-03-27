package net.mullvad.mullvadvpn.lib.model

sealed interface GetLoginTicketError {
    data class Unknown(val error: Throwable) : GetLoginTicketError
}
