package net.mullvad.mullvadvpn.lib.model

interface LogoutAccountError {
    data class Unknown(val t: Throwable) : LogoutAccountError
}
