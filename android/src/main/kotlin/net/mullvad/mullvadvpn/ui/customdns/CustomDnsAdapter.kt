package net.mullvad.mullvadvpn.ui.customdns

import android.support.v7.widget.RecyclerView.Adapter
import android.view.LayoutInflater
import android.view.ViewGroup
import java.net.InetAddress
import kotlin.properties.Delegates.observable
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.service.CustomDns
import net.mullvad.mullvadvpn.util.JobTracker
import org.apache.commons.validator.routines.InetAddressValidator

class CustomDnsAdapter(val customDns: CustomDns) : Adapter<CustomDnsItemHolder>() {
    private enum class ViewTypes {
        ADD_SERVER,
        EDIT_SERVER,
        SHOW_SERVER,
        FOOTER,
    }

    private val inetAddressValidator = InetAddressValidator.getInstance()
    private val jobTracker = JobTracker()

    private var editingPosition: Int? = null

    private var activeCustomDnsServers
    by observable<List<InetAddress>>(emptyList()) { _, _, servers ->
        if (servers != cachedCustomDnsServers) {
            cachedCustomDnsServers = servers.toMutableList()
            notifyDataSetChanged()
        }
    }

    private var cachedCustomDnsServers = emptyList<InetAddress>().toMutableList()

    private var enabled by observable(false) { _, oldValue, newValue ->
        if (oldValue != newValue) {
            notifyDataSetChanged()
        }
    }

    init {
        customDns.apply {
            onDnsServersChanged.subscribe(this) { dnsServers ->
                jobTracker.newUiJob("updateDnsServers") {
                    activeCustomDnsServers = dnsServers
                }
            }

            onEnabledChanged.subscribe(this) { value ->
                jobTracker.newUiJob("updateEnabled") {
                    enabled = value
                }
            }
        }
    }

    override fun getItemCount() =
        if (enabled) {
            cachedCustomDnsServers.size + 2
        } else {
            1
        }

    override fun getItemViewType(position: Int): Int {
        val count = getItemCount()
        val footer = count - 1
        val addServer = count - 2

        if (position == footer) {
            return ViewTypes.FOOTER.ordinal
        } else if (position == editingPosition) {
            return ViewTypes.EDIT_SERVER.ordinal
        } else if (position == addServer) {
            return ViewTypes.ADD_SERVER.ordinal
        } else {
            return ViewTypes.SHOW_SERVER.ordinal
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
                return AddCustomDnsServerHolder(view, this)
            }
            ViewTypes.EDIT_SERVER -> {
                val view = inflater.inflate(R.layout.edit_custom_dns_server, parentView, false)
                return EditCustomDnsServerHolder(view, this)
            }
            ViewTypes.SHOW_SERVER -> {
                val view = inflater.inflate(R.layout.custom_dns_server, parentView, false)
                return CustomDnsServerHolder(view, this)
            }
        }
    }

    override fun onBindViewHolder(holder: CustomDnsItemHolder, position: Int) {
        if (holder is CustomDnsServerHolder) {
            holder.serverAddress = cachedCustomDnsServers[position]
        }
    }

    fun onDestroy() {
        customDns.apply {
            onDnsServersChanged.unsubscribe(this)
            onEnabledChanged.unsubscribe(this)
        }
    }

    fun newDnsServer() {
        if (enabled) {
            val count = getItemCount()

            editingPosition = count - 2

            notifyItemChanged(count - 2)
        }
    }

    fun addDnsServer(address: String) {
        jobTracker.newUiJob("addDnsServer $address") {
            var added = false

            jobTracker.runOnBackground {
                if (inetAddressValidator.isValid(address)) {
                    val address = InetAddress.getByName(address)

                    if (customDns.addDnsServer(address)) {
                        cachedCustomDnsServers.add(address)
                        added = true
                    }
                }
            }

            if (added) {
                editingPosition = null

                val count = getItemCount()

                notifyItemChanged(count - 3)
                notifyItemInserted(count - 2)
            }
        }
    }

    fun removeDnsServer(address: InetAddress) {
        jobTracker.newBackgroundJob("removeDnsServer $address") {
            customDns.removeDnsServer(address)
        }
    }
}
