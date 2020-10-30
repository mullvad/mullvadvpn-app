package net.mullvad.mullvadvpn.ui.customdns

import android.support.v7.widget.RecyclerView.Adapter
import android.view.LayoutInflater
import android.view.ViewGroup
import net.mullvad.mullvadvpn.R

class CustomDnsAdapter : Adapter<CustomDnsItemHolder>() {
    override fun getItemCount() = 0

    override fun onCreateViewHolder(parentView: ViewGroup, type: Int): CustomDnsItemHolder {
        val inflater = LayoutInflater.from(parentView.context)
        val view = inflater.inflate(R.layout.app_list_item, parentView, false)

        return CustomDnsItemHolder(view)
    }

    override fun onBindViewHolder(holder: CustomDnsItemHolder, position: Int) {}
}
