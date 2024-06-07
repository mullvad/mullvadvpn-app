package net.mullvad.mullvadvpn.lib.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

@JvmInline @Parcelize value class AccountNumber(val value: String) : Parcelable
