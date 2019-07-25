package net.mullvad.mullvadvpn

import kotlinx.coroutines.launch
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.Job

import android.graphics.drawable.Drawable
import android.view.View
import android.widget.Button

import net.mullvad.mullvadvpn.model.ActionAfterDisconnect
import net.mullvad.mullvadvpn.model.TunnelState
import net.mullvad.mullvadvpn.relaylist.RelayItem

class SwitchLocationButton(val parentView: View) {
    private val button: Button = parentView.findViewById(R.id.switch_location)
    private val chevron: Drawable = button.compoundDrawables[2]

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
    }

    private fun showLocation() {
        val locationName = location?.locationName

        if (locationName == null) {
            showLabel()
        } else {
            button.setText(locationName)
            button.setCompoundDrawables(null, null, chevron, null)
        }
    }
}
