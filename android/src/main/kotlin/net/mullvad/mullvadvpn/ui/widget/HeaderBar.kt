package net.mullvad.mullvadvpn.ui.widget

import android.content.Context
import android.util.AttributeSet
import android.view.Gravity
import android.view.LayoutInflater
import android.view.View
import android.widget.LinearLayout
import androidx.core.content.ContextCompat
import kotlin.properties.Delegates.observable
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.model.TunnelState
import net.mullvad.mullvadvpn.ui.MainActivity
import net.mullvad.mullvadvpn.ui.StatusBarPainter
import net.mullvad.mullvadvpn.ui.paintStatusBar

class HeaderBar @JvmOverloads constructor(
    context: Context,
    attributes: AttributeSet? = null,
    defStyleAttr: Int = 0,
    defStyleRes: Int = 0
) : LinearLayout(context, attributes, defStyleAttr, defStyleRes), StatusBarPainter {
    private val container = LayoutInflater.from(context).inflate(R.layout.header_bar, this)

    private val disabledColor = ContextCompat.getColor(context, android.R.color.transparent)
    private val securedColor = ContextCompat.getColor(context, R.color.green)
    private val unsecuredColor = ContextCompat.getColor(context, R.color.red)

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
        paintStatusBar(backgroundColor)
    }

    init {
        gravity = Gravity.CENTER_VERTICAL
        orientation = HORIZONTAL

        findViewById<View>(R.id.settings).setOnClickListener {
            (context as? MainActivity)?.openSettings()
        }
    }

    override fun onDetachedFromWindow() {
        findViewById<View>(R.id.settings)?.setOnClickListener(null)
        super.onDetachedFromWindow()
    }
}
