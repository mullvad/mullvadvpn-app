package net.mullvad.mullvadvpn.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

sealed class CreateAccountError : Parcelable {
    @Parcelize data class Unknown(val error: Throwable) : CreateAccountError()
}
