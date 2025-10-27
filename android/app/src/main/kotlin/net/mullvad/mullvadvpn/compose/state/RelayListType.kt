package net.mullvad.mullvadvpn.compose.state

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

enum class MultihopRelayListType {
    ENTRY,
    EXIT,
}

sealed interface RelayListType : Parcelable {
    @Parcelize data class Multihop(val multihopRelayListType: MultihopRelayListType) : RelayListType

    @Parcelize data object Single : RelayListType
}
