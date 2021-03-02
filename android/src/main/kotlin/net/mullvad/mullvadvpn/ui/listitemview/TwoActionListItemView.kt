package net.mullvad.mullvadvpn.ui.listitemview

import android.content.Context
import android.view.ViewGroup
import androidx.appcompat.view.ContextThemeWrapper
import net.mullvad.mullvadvpn.R

class TwoActionListItemView(context: Context) :
    ActionListItemView(ContextThemeWrapper(context, R.style.ListItem_Action_Double)) {
    override val layoutRes: Int
        get() = R.layout.list_item_two_action

    init {
        isClickable = false
        isFocusable = false
    }

    private val container: ViewGroup = findViewById(R.id.container_without_widget)

    override fun updateAction() {
        if (itemData.action == null) {
            container.setOnClickListener(null)
            container.isClickable = false
            container.isFocusable = false
        } else {
            container.setOnClickListener(clickListener)
            container.isClickable = true
            container.isFocusable = true
        }
        widgetContainer.setOnClickListener(clickListener)
        widgetContainer.isClickable = true
        widgetContainer.isFocusable = true
    }
}
