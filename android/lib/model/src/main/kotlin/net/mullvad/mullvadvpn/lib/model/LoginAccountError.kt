package net.mullvad.mullvadvpn.lib.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

@Parcelize
sealed class LoginAccountError : Parcelable {
    data object InvalidAccount : LoginAccountError()

    data class MaxDevicesReached(val accountNumber: AccountNumber) : LoginAccountError()

    data object RpcError : LoginAccountError()

    data class Unknown(val error: Throwable) : LoginAccountError()
}
