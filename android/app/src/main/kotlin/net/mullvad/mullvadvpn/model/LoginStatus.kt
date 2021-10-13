package net.mullvad.mullvadvpn.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize
import org.joda.time.DateTime

@Parcelize
data class LoginStatus(
    val account: String,
    val expiry: DateTime?,
    val isNewAccount: Boolean
) : Parcelable {
    val isExpired: Boolean
        get() = expiry != null && expiry.isAfterNow()
}
