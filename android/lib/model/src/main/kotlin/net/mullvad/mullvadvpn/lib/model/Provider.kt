package net.mullvad.mullvadvpn.lib.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize
import kotlinx.parcelize.RawValue

@Parcelize
data class Provider(val providerId: @RawValue ProviderId, val ownership: Set<Ownership>) :
    Parcelable
