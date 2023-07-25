package net.mullvad.mullvadvpn.model

import android.os.Parcelable
import kotlinx.android.parcel.Parcelize

sealed class AccountHistory : Parcelable {
    @Parcelize data class Available(val accountToken: String) : AccountHistory()

    @Parcelize object Missing : AccountHistory()

    fun accountToken() = (this as? Available)?.accountToken
}
