package net.mullvad.mullvadvpn.lib.model

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

val RelayListType.isMultihopEntry
    get() =
        when (this) {
            is RelayListType.Multihop if multihopRelayListType == MultihopRelayListType.ENTRY ->
                true
            else -> false
        }

fun RelayListType?.toFilterTarget(): FilterTarget =
    when (this) {
        is RelayListType.Multihop ->
            when (multihopRelayListType) {
                MultihopRelayListType.ENTRY -> FilterTarget.Entry
                MultihopRelayListType.EXIT -> FilterTarget.Exit
            }
        RelayListType.Single,
        null -> FilterTarget.Exit
    }
