package net.mullvad.mullvadvpn.feature.location.impl

import net.mullvad.mullvadvpn.lib.model.RelayItemId

// Defines
sealed interface UndoChangeMultihopAction {
    data object Enable : UndoChangeMultihopAction

    data object Disable : UndoChangeMultihopAction

    data class DisableAndSetExit(val relayItemId: RelayItemId) : UndoChangeMultihopAction

    data class DisableAndSetEntry(val relayItemId: RelayItemId) : UndoChangeMultihopAction
}
