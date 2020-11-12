package net.mullvad.mullvadvpn.ui.customdns

import android.text.Editable
import android.text.TextWatcher
import android.view.View
import android.view.View.OnFocusChangeListener
import android.widget.EditText
import java.net.InetAddress
import kotlin.properties.Delegates.observable
import net.mullvad.mullvadvpn.R
import net.mullvad.talpid.util.addressString

class EditCustomDnsServerHolder(view: View, adapter: CustomDnsAdapter) : CustomDnsItemHolder(view) {
    private val context = view.context
    private val errorColor = context.getColor(R.color.red)
    private val normalColor = context.getColor(R.color.blue)

    private val input: EditText = view.findViewById<EditText>(R.id.input).apply {
        onFocusChangeListener = OnFocusChangeListener { _, hasFocus ->
            if (!hasFocus) {
                serverAddress?.let { address ->
                    adapter.stopEditing(address)
                }
            }
        }
    }

    private val watcher = object : TextWatcher {
        override fun beforeTextChanged(text: CharSequence, start: Int, count: Int, after: Int) {}

        override fun afterTextChanged(text: Editable) {
            input.setTextColor(normalColor)
            input.removeTextChangedListener(this)
        }

        override fun onTextChanged(text: CharSequence, start: Int, before: Int, count: Int) {}
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
            adapter.saveDnsServer(input.text.toString()) {
                input.apply {
                    setTextColor(errorColor)
                    addTextChangedListener(watcher)
                }
            }
        }
    }
}
