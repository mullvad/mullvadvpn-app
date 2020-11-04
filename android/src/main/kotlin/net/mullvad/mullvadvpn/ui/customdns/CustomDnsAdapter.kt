package net.mullvad.mullvadvpn.ui.customdns

import android.support.v7.widget.RecyclerView.Adapter
import android.view.LayoutInflater
import android.view.ViewGroup
import kotlin.properties.Delegates.observable
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.service.CustomDns
import net.mullvad.mullvadvpn.util.JobTracker

class CustomDnsAdapter(val customDns: CustomDns) : Adapter<CustomDnsItemHolder>() {
    private enum class ViewTypes {
        ADD_SERVER,
        EDIT_SERVER,
        FOOTER,
    }

    private val jobTracker = JobTracker()

    private var enteringNewServer = false

    private var enabled by observable(false) { _, oldValue, newValue ->
        if (oldValue != newValue) {
            notifyDataSetChanged()
        }
    }

    init {
        customDns.onEnabledChanged.subscribe(this) { value ->
            jobTracker.newUiJob("updateEnabled") {
                enabled = value
            }
        }
    }

    override fun getItemCount() =
        if (enabled) {
            2
        } else {
            1
        }

    override fun getItemViewType(position: Int): Int {
        val count = getItemCount()
        val footer = count - 1
        val addServerOrNewServer = count - 2

        if (position == footer) {
            return ViewTypes.FOOTER.ordinal
        } else if (position == addServerOrNewServer) {
            if (enteringNewServer) {
                return ViewTypes.EDIT_SERVER.ordinal
            } else {
                return ViewTypes.ADD_SERVER.ordinal
            }
        } else {
            throw RuntimeException("Too many items in the custom DNS list")
        }
    }

    override fun onCreateViewHolder(parentView: ViewGroup, type: Int): CustomDnsItemHolder {
        val inflater = LayoutInflater.from(parentView.context)

        when (ViewTypes.values()[type]) {
            ViewTypes.FOOTER -> {
                val view = inflater.inflate(R.layout.custom_dns_footer, parentView, false)
                return CustomDnsFooterHolder(view)
            }
            ViewTypes.ADD_SERVER -> {
                val view = inflater.inflate(R.layout.add_custom_dns_server, parentView, false)
                return AddCustomDnsServerHolder(view)
            }
            ViewTypes.EDIT_SERVER -> {
                val view = inflater.inflate(R.layout.edit_custom_dns_server, parentView, false)
                return EditCustomDnsServerHolder(view)
            }
        }
    }

    override fun onBindViewHolder(holder: CustomDnsItemHolder, position: Int) {}

    fun onDestroy() {
        customDns.onEnabledChanged.unsubscribe(this)
    }
}
