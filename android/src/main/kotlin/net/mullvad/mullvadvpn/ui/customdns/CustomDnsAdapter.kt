package net.mullvad.mullvadvpn.ui.customdns

import android.support.v7.widget.RecyclerView.Adapter
import android.view.LayoutInflater
import android.view.ViewGroup
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.service.CustomDns
import net.mullvad.mullvadvpn.util.JobTracker

class CustomDnsAdapter(val customDns: CustomDns) : Adapter<CustomDnsItemHolder>() {
    private val jobTracker = JobTracker()

    private var enabled = false

    init {
        customDns.onEnabledChanged.subscribe(this) { value ->
            jobTracker.newUiJob("updateEnabled") {
                enabled = value
            }
        }
    }

    override fun getItemCount() = 1

    override fun onCreateViewHolder(parentView: ViewGroup, type: Int): CustomDnsItemHolder {
        val inflater = LayoutInflater.from(parentView.context)
        val view = inflater.inflate(R.layout.custom_dns_footer, parentView, false)

        return CustomDnsFooterHolder(view)
    }

    override fun onBindViewHolder(holder: CustomDnsItemHolder, position: Int) {}

    fun onDestroy() {
        customDns.onEnabledChanged.unsubscribe(this)
    }
}
