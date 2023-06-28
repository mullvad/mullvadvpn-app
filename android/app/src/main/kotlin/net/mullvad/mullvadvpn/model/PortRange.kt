package net.mullvad.mullvadvpn.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

@Parcelize data class PortRange(val from: Int, val to: Int) : Parcelable
