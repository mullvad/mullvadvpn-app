package net.mullvad.mullvadvpn.feature.location.api

import android.os.Parcelable
import kotlinx.parcelize.Parcelize
import net.mullvad.mullvadvpn.lib.model.RelayItemId

@Parcelize
sealed interface UndoChangeMultihopAction : Parcelable {
    data object Enable : UndoChangeMultihopAction

    data object Disable : UndoChangeMultihopAction

    data class DisableAndSetExit(val relayItemId: RelayItemId) : UndoChangeMultihopAction

    data class DisableAndSetEntry(val relayItemId: RelayItemId) : UndoChangeMultihopAction
}
