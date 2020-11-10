package net.mullvad.mullvadvpn.ui.customdns

import android.view.View
import android.widget.TextView
import net.mullvad.mullvadvpn.R

class EditCustomDnsServerHolder(view: View, adapter: CustomDnsAdapter) : CustomDnsItemHolder(view) {
    private val input: TextView = view.findViewById(R.id.input)

    init {
        view.findViewById<View>(R.id.save).setOnClickListener {
            adapter.saveDnsServer(input.text.toString())
        }
    }
}
