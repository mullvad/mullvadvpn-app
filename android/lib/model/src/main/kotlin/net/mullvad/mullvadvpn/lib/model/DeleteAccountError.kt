package net.mullvad.mullvadvpn.lib.model

sealed interface DeleteAccountError {
    data class Unknown(val t: Throwable) : DeleteAccountError
}
