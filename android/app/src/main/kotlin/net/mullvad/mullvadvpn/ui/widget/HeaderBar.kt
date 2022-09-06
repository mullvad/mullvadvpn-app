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
    private val settingsButton = findViewById<View>(R.id.settings)

    private val disabledColor = ContextCompat.getColor(context, android.R.color.transparent)
    private val securedColor = ContextCompat.getColor(context, R.color.green)
    private val unsecuredColor = ContextCompat.getColor(context, R.color.red)

    var tunnelState by observable<TunnelState?>(null) { _, _, state ->
        val backgroundColor = if (state == null) {
            disabledColor
        } else if (state.isSecured()) {
            securedColor
        } else {
            unsecuredColor
        }

        container.setBackgroundColor(backgroundColor)
        paintStatusBar(backgroundColor)
    }

    init {
        gravity = Gravity.CENTER_VERTICAL
        orientation = HORIZONTAL

        settingsButton.apply {
            isEnabled = true
            setOnClickListener {
                (context as? MainActivity)?.openSettings()
            }
        }

        tunnelState = null
    }

    fun setSettingsButtonEnabled(isEnabled: Boolean) {
        settingsButton.isEnabled = isEnabled
    }
}
