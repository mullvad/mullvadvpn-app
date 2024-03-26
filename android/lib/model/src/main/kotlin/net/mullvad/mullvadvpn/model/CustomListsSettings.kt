package net.mullvad.mullvadvpn.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

@Parcelize data class CustomListsSettings(val customLists: List<CustomList>) : Parcelable
