package net.mullvad.mullvadvpn.ui.listitemview

import android.content.Context
import androidx.appcompat.view.ContextThemeWrapper
import net.mullvad.mullvadvpn.R

class DividerGroupListItemView(context: Context) :
    ListItemView(ContextThemeWrapper(context, R.style.ListItem_DividerGroup)) {

    override val layoutRes: Int
        get() = R.layout.list_item_group_divider

    override val heightRes: Int
        get() = R.dimen.vertical_space
}
