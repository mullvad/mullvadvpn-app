package net.mullvad.mullvadvpn.lib.payment.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

@JvmInline @Parcelize value class ProductId(val value: String) : Parcelable
