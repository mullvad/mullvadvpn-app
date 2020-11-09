package net.mullvad.mullvadvpn.ui.customdns

import android.view.View
import android.widget.TextView
import java.net.InetAddress
import kotlin.properties.Delegates.observable
import net.mullvad.mullvadvpn.R

class CustomDnsServerHolder(view: View) : CustomDnsItemHolder(view) {
    private val label: TextView = view.findViewById(R.id.label)

    var serverAddress by observable<InetAddress?>(null) { _, _, address ->
        label.text = address.toString()
    }
}
