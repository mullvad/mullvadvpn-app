package net.mullvad.mullvadvpn.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize
import org.joda.time.DateTime

@Parcelize
data class RedeemVoucherSuccess(val timeAdded: Long, val newExpiry: DateTime) : Parcelable
