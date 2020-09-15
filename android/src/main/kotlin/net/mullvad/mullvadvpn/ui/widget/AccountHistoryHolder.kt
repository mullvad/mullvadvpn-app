package net.mullvad.mullvadvpn.ui.widget

import android.support.v7.widget.RecyclerView.ViewHolder
import android.view.View
import android.widget.TextView
import kotlin.properties.Delegates.observable
import net.mullvad.mullvadvpn.R

class AccountHistoryHolder(view: View) : ViewHolder(view) {
    private val label: TextView = view.findViewById(R.id.label)

    var accountToken by observable("") { _, _, account ->
        label.text = account
    }

    var onSelect: ((String) -> Unit)? = null

    init {
        view.setOnClickListener {
            onSelect?.invoke(accountToken)
        }
    }
}
