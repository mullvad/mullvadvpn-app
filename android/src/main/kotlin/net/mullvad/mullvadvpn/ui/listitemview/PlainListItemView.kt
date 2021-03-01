package net.mullvad.mullvadvpn.ui.listitemview

import android.content.Context
import android.widget.TextView
import androidx.appcompat.view.ContextThemeWrapper
import net.mullvad.mullvadvpn.R

class PlainListItemView(context: Context) :
    ListItemView(ContextThemeWrapper(context, R.style.ListItem_PlainText)) {
    override val layoutRes: Int
        get() = R.layout.list_item_plain_text
    override val heightRes: Int? = null
    private val plainText: TextView = findViewById(R.id.plain_text)

    override fun onUpdate() {
        updateText()
    }

    private fun updateText() {
        itemData.textRes?.let {
            plainText.setText(it)
            return
        }
        itemData.text?.let {
            plainText.text = it
            return
        }
        plainText.text = ""
    }
}
