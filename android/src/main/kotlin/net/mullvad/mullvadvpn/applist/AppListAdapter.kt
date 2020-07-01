package net.mullvad.mullvadvpn.applist

import android.support.v7.widget.RecyclerView.Adapter
import android.view.LayoutInflater
import android.view.ViewGroup
import net.mullvad.mullvadvpn.R

class AppListAdapter : Adapter<AppListHolder>() {
    override fun onCreateViewHolder(parentView: ViewGroup, type: Int): AppListHolder {
        val inflater = LayoutInflater.from(parentView.context)
        val view = inflater.inflate(R.layout.app_list_item, parentView, false)

        return AppListHolder(view)
    }

    override fun onBindViewHolder(holder: AppListHolder, position: Int) {}
    override fun getItemCount() = 0
}
