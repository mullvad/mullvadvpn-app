package net.mullvad.mullvadvpn.ui.widget

import android.support.v7.widget.RecyclerView.Adapter
import android.view.LayoutInflater
import android.view.ViewGroup
import kotlin.properties.Delegates.observable
import net.mullvad.mullvadvpn.R

class AccountHistoryAdapter : Adapter<AccountHistoryHolder>() {
    var accountHistory by observable(ArrayList<String>()) { _, _, _ ->
        notifyDataSetChanged()
    }

    var onSelectEntry: ((String) -> Unit)? = null

    override fun onCreateViewHolder(parentView: ViewGroup, type: Int): AccountHistoryHolder {
        val inflater = LayoutInflater.from(parentView.context)
        val view = inflater.inflate(R.layout.account_history_entry, parentView, false)

        return AccountHistoryHolder(view).apply {
            onSelect = { account -> onSelectEntry?.invoke(account) }
        }
    }

    override fun onBindViewHolder(holder: AccountHistoryHolder, position: Int) {
        holder.accountToken = accountHistory[position]
    }

    override fun getItemCount() = accountHistory.size
}
