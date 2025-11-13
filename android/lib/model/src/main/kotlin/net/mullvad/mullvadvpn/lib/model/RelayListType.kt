package net.mullvad.mullvadvpn.lib.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

@Parcelize
enum class MultihopRelayListType : Parcelable {
    ENTRY,
    EXIT,
}

sealed interface RelayListType : Parcelable {
    @Parcelize data class Multihop(val multihopRelayListType: MultihopRelayListType) : RelayListType

    @Parcelize data object Single : RelayListType
}

fun RelayListType.isMultihopEntry(): Boolean =
    this is RelayListType.Multihop && this.multihopRelayListType == MultihopRelayListType.ENTRY
