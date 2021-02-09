package net.mullvad.mullvadvpn.ui.listitemview

import android.content.Context
import androidx.appcompat.view.ContextThemeWrapper
import net.mullvad.mullvadvpn.R

class ProgressListItemView(context: Context) :
    ListItemView(ContextThemeWrapper(context, R.style.ListItem_DividerGroup)) {

    override val layoutRes: Int
        get() = R.layout.list_item_progress

    override val heightRes: Int
        get() = R.dimen.progress_size
}
