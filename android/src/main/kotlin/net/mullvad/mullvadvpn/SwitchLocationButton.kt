package net.mullvad.mullvadvpn

import android.view.View
import android.widget.Button

class SwitchLocationButton(val parentView: View) {
    private val button: Button = parentView.findViewById(R.id.switch_location)

    var onClick: (() -> Unit)? = null

    init {
        button.setOnClickListener { onClick?.invoke() }
    }
}
