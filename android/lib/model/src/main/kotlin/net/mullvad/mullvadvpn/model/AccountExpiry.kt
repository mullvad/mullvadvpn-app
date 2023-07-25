package net.mullvad.mullvadvpn.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize
import org.joda.time.DateTime

sealed class AccountExpiry : Parcelable {
    @Parcelize data class Available(val expiryDateTime: DateTime) : AccountExpiry()

    @Parcelize object Missing : AccountExpiry()

    fun date(): DateTime? {
        return (this as? Available)?.expiryDateTime
    }
}
