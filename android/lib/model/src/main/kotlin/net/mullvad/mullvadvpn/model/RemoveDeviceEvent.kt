package net.mullvad.mullvadvpn.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

@Parcelize
data class RemoveDeviceEvent(val accountToken: String, val newDevices: List<Device>) : Parcelable
