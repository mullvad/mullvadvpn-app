package net.mullvad.mullvadvpn.lib.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

@Parcelize @JvmInline value class Cipher(val value: String) : Parcelable
