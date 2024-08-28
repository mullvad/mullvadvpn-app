package net.mullvad.mullvadvpn.lib.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

@Parcelize
data class NewAccessMethodSetting(
    val name: ApiAccessMethodName,
    val enabled: Boolean,
    val apiAccessMethod: ApiAccessMethod.CustomProxy,
) : Parcelable
