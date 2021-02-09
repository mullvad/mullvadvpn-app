package net.mullvad.mullvadvpn.ui.listitemview

import android.content.Context
import androidx.appcompat.view.ContextThemeWrapper
import kotlinx.android.synthetic.main.list_item_base.view.*
// import androidx.appcompat.widget.AppCompatTextView
import kotlinx.android.synthetic.main.list_item_plain_text.view.*
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.model.ListItemData

class PlainListItemView(context: Context) :
    ListItemView(ContextThemeWrapper(context, R.style.ListItem_PlainText)) {

    override val layoutRes: Int
        get() = R.layout.list_item_plain_text

    override val heightRes: Int
        get() = 0

    override fun update(data: ListItemData) {
        super.update(data)
        updateText()
    }

    private fun updateText() {
        itemData.textRes?.let {
            plain_text.setText(it)
            return
        }
        itemData.text?.let {
            plain_text.setText(it)
            return
        }
        plain_text.text = ""
    }
}
