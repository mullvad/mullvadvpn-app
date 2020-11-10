package net.mullvad.mullvadvpn.ui.customdns

import android.view.View
import android.widget.TextView
import java.net.InetAddress
import kotlin.properties.Delegates.observable
import net.mullvad.mullvadvpn.R
import net.mullvad.talpid.util.addressString

class EditCustomDnsServerHolder(view: View, adapter: CustomDnsAdapter) : CustomDnsItemHolder(view) {
    private val input: TextView = view.findViewById(R.id.input)

    var serverAddress by observable<InetAddress?>(null) { _, _, address ->
        input.text = address?.addressString() ?: ""
    }

    init {
        view.findViewById<View>(R.id.save).setOnClickListener {
            adapter.saveDnsServer(input.text.toString())
        }
    }
}
