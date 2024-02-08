package net.mullvad.mullvadvpn.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

@Parcelize
data class CustomList(
    val id: String,
    val name: String,
    val locations: ArrayList<GeographicLocationConstraint>
) : Parcelable
