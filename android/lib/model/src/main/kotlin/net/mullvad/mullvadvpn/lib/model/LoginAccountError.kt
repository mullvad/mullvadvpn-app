package net.mullvad.mullvadvpn.lib.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

@Parcelize
sealed class LoginAccountError : Parcelable {
    data object InvalidAccount : LoginAccountError()

    data class MaxDevicesReached(val accountNumber: AccountNumber) : LoginAccountError()

    data class InvalidInput(val accountNumber: AccountNumber) : LoginAccountError()

    data object TooManyAttempts : LoginAccountError()

    data object Timeout : LoginAccountError()

    data object ApiUnreachable : LoginAccountError()

    data class Unknown(val error: Throwable) : LoginAccountError()
}
