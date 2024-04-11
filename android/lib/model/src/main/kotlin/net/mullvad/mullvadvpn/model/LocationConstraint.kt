package net.mullvad.mullvadvpn.model

import android.os.Parcelable
import arrow.optics.optics
import kotlinx.parcelize.Parcelize

@optics
sealed class LocationConstraint : Parcelable {
    @Parcelize
    @optics
    data class Location(val location: GeographicLocationConstraint) : LocationConstraint() {
        companion object
    }

    @Parcelize
    @optics
    data class CustomList(val listId: CustomListId) : LocationConstraint() {
        companion object
    }

    companion object
}
