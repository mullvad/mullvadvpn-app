package net.mullvad.mullvadvpn.applist

import net.mullvad.mullvadvpn.model.ListItemData

sealed class ViewIntent {
    // In future we will have search intent
    data class ChangeApplicationGroup(val item: ListItemData) : ViewIntent()
    object ViewIsReady : ViewIntent()
    data class ShowSystemApps(internal val show: Boolean) : ViewIntent()
}
