package net.mullvad.mullvadvpn.ui.listitemview

import android.content.Context
import androidx.appcompat.view.ContextThemeWrapper
import net.mullvad.mullvadvpn.R

class TwoActionListItemView(context: Context) :
    ActionListItemView(ContextThemeWrapper(context, R.style.ListItem_Action_Double)) {

    override val layoutRes: Int
        get() = R.layout.list_item_two_action
}
