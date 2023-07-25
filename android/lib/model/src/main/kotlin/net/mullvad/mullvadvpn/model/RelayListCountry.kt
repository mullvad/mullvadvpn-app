package net.mullvad.mullvadvpn.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

@Parcelize
data class RelayListCountry(
    val name: String,
    val code: String,
    val cities: ArrayList<RelayListCity>
) : Parcelable
