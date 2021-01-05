package net.mullvad.mullvadvpn.util

import android.os.Parcelable
import java.net.InetAddress
import kotlinx.parcelize.Parcelize

@Parcelize
data class ParcelableInetAddress(val address: InetAddress) : Parcelable
