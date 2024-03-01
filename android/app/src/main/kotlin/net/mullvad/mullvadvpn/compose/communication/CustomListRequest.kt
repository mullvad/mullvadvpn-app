package net.mullvad.mullvadvpn.compose.communication

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

@Parcelize @JvmInline value class CustomListRequest(val action: CustomListAction) : Parcelable

inline fun <reified T> CustomListRequest.parsedAction(): T = action as T
