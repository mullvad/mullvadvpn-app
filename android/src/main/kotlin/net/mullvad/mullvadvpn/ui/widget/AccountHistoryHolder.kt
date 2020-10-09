package net.mullvad.mullvadvpn.ui.widget

import android.support.v7.widget.RecyclerView.ViewHolder
import android.view.View
import android.widget.TextView
import kotlin.properties.Delegates.observable
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.util.SegmentedTextFormatter

class AccountHistoryHolder(
    view: View,
    private val formatter: SegmentedTextFormatter
) : ViewHolder(view) {
    private val label: TextView = view.findViewById(R.id.label)

    var accountToken by observable("") { _, _, account ->
        label.text = formatter.format(account)
    }

    var onSelect: ((String) -> Unit)? = null
    var onRemove: ((String) -> Unit)? = null

    init {
        label.setOnClickListener {
            onSelect?.invoke(accountToken)
        }

        view.findViewById<View>(R.id.remove).setOnClickListener {
            onRemove?.invoke(accountToken)
        }
    }
}
