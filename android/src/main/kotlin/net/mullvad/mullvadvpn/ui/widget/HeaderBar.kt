package net.mullvad.mullvadvpn.ui.widget

import android.content.Context
import android.util.AttributeSet
import android.view.Gravity
import android.view.LayoutInflater
import android.view.View
import android.widget.LinearLayout
import kotlin.properties.Delegates.observable
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.model.TunnelState
import net.mullvad.mullvadvpn.ui.MainActivity

class HeaderBar : LinearLayout {
    private val container =
        context.getSystemService(Context.LAYOUT_INFLATER_SERVICE).let { service ->
            val inflater = service as LayoutInflater

            inflater.inflate(R.layout.header_bar, this)
        }

    private val disabledColor = context.getColor(android.R.color.transparent)
    private val securedColor = context.getColor(R.color.green)
    private val unsecuredColor = context.getColor(R.color.red)

    var tunnelState by observable<TunnelState?>(null) { _, _, state ->
        val backgroundColor = when (state) {
            null -> disabledColor
            is TunnelState.Disconnected -> unsecuredColor
            is TunnelState.Connecting -> securedColor
            is TunnelState.Connected -> securedColor
            is TunnelState.Disconnecting -> securedColor
            is TunnelState.Error -> {
                if (state.errorState.isBlocking) {
                    securedColor
                } else {
                    unsecuredColor
                }
            }
        }

        container.setBackgroundColor(backgroundColor)
    }

    constructor(context: Context) : super(context) {}

    constructor(context: Context, attributes: AttributeSet) : super(context, attributes) {}

    constructor(
        context: Context,
        attributes: AttributeSet,
        defaultStyleAttribute: Int
    ) : super(context, attributes, defaultStyleAttribute) {}

    constructor(
        context: Context,
        attributes: AttributeSet,
        defaultStyleAttribute: Int,
        defaultStyleResource: Int
    ) : super(context, attributes, defaultStyleAttribute, defaultStyleResource) {}

    init {
        gravity = Gravity.CENTER_VERTICAL
        orientation = HORIZONTAL

        findViewById<View>(R.id.settings).setOnClickListener {
            (context as? MainActivity)?.openSettings()
        }
    }
}
