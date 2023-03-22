package net.mullvad.mullvadvpn.ui.widget

import android.view.View
import android.widget.TextView
import androidx.recyclerview.widget.RecyclerView.ViewHolder
import kotlin.properties.Delegates.observable
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.util.SegmentedTextFormatter

class AccountHistoryHolder(view: View, private val formatter: SegmentedTextFormatter) :
    ViewHolder(view) {
    private val label: TextView = view.findViewById(R.id.label)

    var accountToken by observable("") { _, _, account -> label.text = formatter.format(account) }

    var onSelect: ((String) -> Unit)? = null
    var onRemove: ((String) -> Unit)? = null
    var onFocusChanged: ((String, Boolean) -> Unit)? = null

    init {
        view.findViewById<View>(R.id.remove).apply {
            setOnClickListener { onRemove?.invoke(accountToken) }

            setOnFocusChangeListener { _, hasFocus ->
                onFocusChanged?.invoke(accountToken, hasFocus)
            }
        }

        label.apply {
            setOnClickListener { onSelect?.invoke(accountToken) }

            setOnFocusChangeListener { _, hasFocus ->
                onFocusChanged?.invoke(accountToken, hasFocus)
            }
        }
    }
}
