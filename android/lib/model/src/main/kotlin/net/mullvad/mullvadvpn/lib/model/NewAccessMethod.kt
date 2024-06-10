package net.mullvad.mullvadvpn.lib.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

@Parcelize
data class NewAccessMethod(
    val name: ApiAccessMethodName,
    val enabled: Boolean,
    val apiAccessMethodType: ApiAccessMethodType.CustomProxy
) : Parcelable
