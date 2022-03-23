package net.mullvad.mullvadvpn.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

@Parcelize
data class Device(
    val id: String,
    val name: String,
    val pubkey: ByteArray,
    val ports: ArrayList<String>
) : Parcelable
