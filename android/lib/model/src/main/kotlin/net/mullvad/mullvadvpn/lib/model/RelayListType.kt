package net.mullvad.mullvadvpn.lib.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

sealed interface RelayListType : Parcelable {
    @Parcelize data class Multihop(val hopType: RelayHopType) : RelayListType

    @Parcelize data object Single : RelayListType
}

val RelayListType.isMultihopEntry
    get() =
        when (this) {
            is RelayListType.Multihop if hopType == RelayHopType.ENTRY ->
                true
            else -> false
        }

fun RelayListType.hopType(): RelayHopType =
    when (this) {
        is RelayListType.Multihop -> hopType
        RelayListType.Single -> RelayHopType.EXIT
    }
