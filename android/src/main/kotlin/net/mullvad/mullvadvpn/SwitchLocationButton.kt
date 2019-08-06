package net.mullvad.mullvadvpn

import kotlinx.coroutines.launch
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.Job

import android.content.res.Resources
import android.graphics.drawable.Drawable
import android.text.TextUtils.TruncateAt
import android.view.View
import android.view.ViewGroup.MarginLayoutParams
import android.widget.Button

import net.mullvad.mullvadvpn.model.ActionAfterDisconnect
import net.mullvad.mullvadvpn.model.TunnelState
import net.mullvad.mullvadvpn.relaylist.RelayItem

class SwitchLocationButton(val parentView: View, val resources: Resources) {
    private val button: Button = parentView.findViewById(R.id.switch_location)
    private val chevron: Drawable = button.compoundDrawables[2]

    private val normalButtonHeight = resources.getDimensionPixelSize(R.dimen.normal_button_height)
    private val tallButtonHeight = resources.getDimensionPixelSize(R.dimen.tall_button_height)
    private val topMargin = tallButtonHeight - normalButtonHeight

    private var tall = false
    private var updateJob: Job? = null

    var location: RelayItem? = null
        set(value) {
            field = value
            update()
        }

    var state: TunnelState = TunnelState.Disconnected()
        set(value) {
            field = value
            update()
        }

    var onClick: (() -> Unit)? = null

    init {
        button.setOnClickListener { onClick?.invoke() }
        button.addOnLayoutChangeListener { _, _, _, _, _, _, _, _, _ -> resizeIfNecessary() }
    }

    fun onDestroy() {
        updateJob?.cancel()
    }

    private fun update() {
        updateJob?.cancel()
        updateJob = GlobalScope.launch(Dispatchers.Main) {
            val state = this@SwitchLocationButton.state

            when (state) {
                is TunnelState.Disconnected -> showLocation()
                is TunnelState.Disconnecting -> {
                    when (state.actionAfterDisconnect) {
                        is ActionAfterDisconnect.Nothing -> showLocation()
                        is ActionAfterDisconnect.Block -> showLocation()
                        is ActionAfterDisconnect.Reconnect -> showLabel()
                    }
                }
                is TunnelState.Connecting -> showLabel()
                is TunnelState.Connected -> showLabel()
                is TunnelState.Blocked -> showLocation()
            }
        }
    }

    private fun showLabel() {
        button.setText(R.string.switch_location)
        button.setCompoundDrawables(null, null, null, null)
        resizeIfNecessary()
    }

    private fun showLocation() {
        val locationName = location?.locationName

        if (locationName == null) {
            showLabel()
        } else {
            button.setText(locationName)
            button.setCompoundDrawables(null, null, chevron, null)
            resizeIfNecessary()
        }
    }

    private fun resizeIfNecessary() {
        val layoutParams = button.layoutParams

        if (button.lineCount > 1 && !tall) {
            tall = true

            if (layoutParams is MarginLayoutParams) {
                layoutParams.height = tallButtonHeight
                layoutParams.topMargin = 0
            }

            button.maxLines = 2
            button.ellipsize = TruncateAt.END
            button.requestLayout()
        } else if (button.lineCount <= 1 && tall) {
            tall = false

            if (layoutParams is MarginLayoutParams) {
                layoutParams.height = normalButtonHeight
                layoutParams.topMargin = topMargin
            }

            button.maxLines = -1
            button.ellipsize = null
            button.requestLayout()
        }
    }
}
