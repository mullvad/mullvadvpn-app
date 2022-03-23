package net.mullvad.mullvadvpn.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

@Parcelize
data class DeviceConfig(
    val token: String,
    val device: Device
) : Parcelable
