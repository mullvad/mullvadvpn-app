package net.mullvad.mullvadvpn.ui.widget

import android.content.Context
import android.util.AttributeSet
import android.view.LayoutInflater
import android.view.View
import android.widget.FrameLayout
import android.widget.TextView
import kotlin.properties.Delegates.observable
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.model.TunnelState
import net.mullvad.mullvadvpn.relaylist.RelayItem
import net.mullvad.talpid.tunnel.ActionAfterDisconnect

class SwitchLocationButton : FrameLayout {
    private val container =
        context.getSystemService(Context.LAYOUT_INFLATER_SERVICE).let { service ->
            val inflater = service as LayoutInflater

            inflater.inflate(R.layout.switch_location_button, this)
        }

    private val buttonWithLabel =
        container.findViewById<View>(R.id.button_with_label).apply {
            setOnClickListener { onClick?.invoke() }
        }

    private val buttonWithLocation =
        container.findViewById<TextView>(R.id.button_with_location).apply {
            setOnClickListener { onClick?.invoke() }
        }

    var onClick: (() -> Unit)? = null

    var location by
        observable<RelayItem?>(null) { _, _, location ->
            buttonWithLocation.text = location?.locationName ?: ""
        }

    var tunnelState by
        observable<TunnelState>(TunnelState.Disconnected) { _, _, state ->
            when (state) {
                is TunnelState.Disconnected -> showLocation()
                is TunnelState.Disconnecting -> {
                    when (state.actionAfterDisconnect) {
                        ActionAfterDisconnect.Nothing -> showLocation()
                        ActionAfterDisconnect.Block -> showLocation()
                        ActionAfterDisconnect.Reconnect -> showLabel()
                    }
                }
                is TunnelState.Connecting -> showLabel()
                is TunnelState.Connected -> showLabel()
                is TunnelState.Error -> showLocation()
            }
        }

    constructor(context: Context) : super(context)

    constructor(context: Context, attributes: AttributeSet) : super(context, attributes)

    constructor(
        context: Context,
        attributes: AttributeSet,
        defaultStyleAttribute: Int
    ) : super(context, attributes, defaultStyleAttribute)

    private fun showLabel() {
        updateButton(buttonWithLabel, true)
        updateButton(buttonWithLocation, false)
    }

    private fun showLocation() {
        updateButton(buttonWithLabel, false)
        updateButton(buttonWithLocation, true)
    }

    private fun updateButton(button: View, show: Boolean) {
        button.apply {
            setEnabled(show)

            visibility =
                if (show) {
                    View.VISIBLE
                } else {
                    View.INVISIBLE
                }
        }
    }
}
