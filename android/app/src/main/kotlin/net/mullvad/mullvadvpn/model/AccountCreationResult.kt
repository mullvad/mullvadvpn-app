package net.mullvad.mullvadvpn.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

sealed class AccountCreationResult : Parcelable {
    @Parcelize
    data class Success(val accountToken: String) : AccountCreationResult()

    @Parcelize
    object Failure : AccountCreationResult()
}
