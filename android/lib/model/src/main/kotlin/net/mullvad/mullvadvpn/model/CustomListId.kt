package net.mullvad.mullvadvpn.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

@Parcelize @JvmInline value class CustomListId(val value: String) : Parcelable
