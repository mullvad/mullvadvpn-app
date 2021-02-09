package net.mullvad.mullvadvpn.ui

import net.mullvad.mullvadvpn.model.ListItemData

interface ListItemListener {
    fun onItemAction(item: ListItemData)
}
