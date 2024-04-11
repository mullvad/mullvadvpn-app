package net.mullvad.mullvadvpn.model

import android.os.Parcelable
import arrow.optics.optics
import kotlinx.parcelize.Parcelize

@Parcelize
enum class Ownership : Parcelable {
    MullvadOwned,
    Rented
}
