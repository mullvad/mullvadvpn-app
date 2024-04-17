package net.mullvad.mullvadvpn.model

import android.os.Parcelable
import arrow.optics.optics
import kotlinx.parcelize.Parcelize

@Parcelize
@optics
data class CustomList(
    val id: CustomListId,
    val name: CustomListName,
    val locations: List<GeoLocationId>
) : Parcelable {
    companion object
}
