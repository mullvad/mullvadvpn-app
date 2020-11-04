package net.mullvad.mullvadvpn.ui.customdns

import android.view.View
import net.mullvad.mullvadvpn.R

class AddCustomDnsServerHolder(view: View, adapter: CustomDnsAdapter) : CustomDnsItemHolder(view) {
    init {
        view.findViewById<View>(R.id.add).setOnClickListener {
            adapter.newDnsServer()
        }

        view.findViewById<View>(R.id.click_area).setOnClickListener {
            adapter.newDnsServer()
        }
    }
}
