package net.mullvad.mullvadvpn.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

@Parcelize data class PublicKey(val key: ByteArray, val dateCreated: String) : Parcelable
