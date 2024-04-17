package net.mullvad.mullvadvpn.model

import android.os.Parcelable
import arrow.optics.optics
import kotlinx.parcelize.Parcelize

@Parcelize
@optics
data class Providers(val providers: Set<ProviderId>) : Parcelable {
    companion object
}
