package net.mullvad.mullvadvpn.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

@Parcelize @JvmInline value class ProviderId(val value: String) : Parcelable
