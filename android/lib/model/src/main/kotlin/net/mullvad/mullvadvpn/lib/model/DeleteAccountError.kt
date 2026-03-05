package net.mullvad.mullvadvpn.lib.model

sealed interface DeleteAccountError {
    data class Unknown(val t: Throwable) : DeleteAccountError

    data class UnableToReachApi(val t: Throwable) : DeleteAccountError

    // 400
    object AccountNumberDoesNotMatch : DeleteAccountError
}
