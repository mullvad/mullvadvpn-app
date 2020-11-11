package net.mullvad.mullvadvpn.ui.customdns

import android.view.View
import android.view.View.OnFocusChangeListener
import android.widget.EditText
import java.net.InetAddress
import kotlin.properties.Delegates.observable
import net.mullvad.mullvadvpn.R
import net.mullvad.talpid.util.addressString

class EditCustomDnsServerHolder(view: View, adapter: CustomDnsAdapter) : CustomDnsItemHolder(view) {
    private val input: EditText = view.findViewById<EditText>(R.id.input).apply {
        onFocusChangeListener = OnFocusChangeListener { _, hasFocus ->
            if (!hasFocus) {
                serverAddress?.let { address ->
                    adapter.stopEditing(address)
                }
            }
        }
    }

    var serverAddress by observable<InetAddress?>(null) { _, _, address ->
        if (address != null) {
            val addressString = address.addressString()

            input.setText(addressString)
            input.setSelection(addressString.length)
        } else {
            input.setText("")
        }

        input.requestFocus()
    }

    init {
        view.findViewById<View>(R.id.save).setOnClickListener {
            adapter.saveDnsServer(input.text.toString())
        }
    }
}
