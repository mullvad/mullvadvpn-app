package net.mullvad.mullvadvpn.ui.widget

import android.support.v7.widget.RecyclerView.Adapter
import android.view.LayoutInflater
import android.view.ViewGroup
import kotlin.properties.Delegates.observable
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.util.SegmentedTextFormatter

class AccountHistoryAdapter : Adapter<AccountHistoryHolder>() {
    private val formatter = SegmentedTextFormatter(' ').apply {
        isValidInputCharacter = { character ->
            '0' <= character && character <= '9'
        }
    }

    var accountHistory by observable(ArrayList<String>()) { _, _, _ ->
        notifyDataSetChanged()
    }

    var onSelectEntry: ((String) -> Unit)? = null
    var onRemoveEntry: ((String) -> Unit)? = null

    override fun onCreateViewHolder(parentView: ViewGroup, type: Int): AccountHistoryHolder {
        val inflater = LayoutInflater.from(parentView.context)
        val view = inflater.inflate(R.layout.account_history_entry, parentView, false)

        return AccountHistoryHolder(view, formatter).apply {
            onSelect = { account -> onSelectEntry?.invoke(account) }
            onRemove = { account -> onRemoveEntry?.invoke(account) }
        }
    }

    override fun onBindViewHolder(holder: AccountHistoryHolder, position: Int) {
        holder.accountToken = accountHistory[position]
    }

    override fun getItemCount() = accountHistory.size
}
