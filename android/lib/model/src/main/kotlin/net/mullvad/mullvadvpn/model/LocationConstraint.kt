package net.mullvad.mullvadvpn.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

sealed class LocationConstraint : Parcelable {
    @Parcelize
    data class Location(val location: GeographicLocationConstraint) : LocationConstraint()

    @Parcelize data class CustomList(val listId: CustomListId) : LocationConstraint()
}
