package net.mullvad.mullvadvpn.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

@JvmInline @Parcelize value class Port(val value: Int) : Parcelable
