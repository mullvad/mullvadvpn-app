package net.mullvad.mullvadvpn.ui

import net.mullvad.mullvadvpn.applist.ListItemData

interface ListItemListener {
    fun onItemAction(item: ListItemData)
}
