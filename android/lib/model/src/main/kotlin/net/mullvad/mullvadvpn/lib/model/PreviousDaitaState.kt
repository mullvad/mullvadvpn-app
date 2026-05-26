package net.mullvad.mullvadvpn.lib.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

@Parcelize
enum class PreviousDaitaState : Parcelable {
    ON,
    DIRECT_ONLY,
    OFF,
}
