package net.mullvad.mullvadvpn.lib.model

interface DeleteAccountError {
    data class Unknown(val t: Throwable) : DeleteAccountError
}
