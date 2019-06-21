package net.mullvad.mullvadvpn

import kotlinx.coroutines.launch
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.Job

import android.view.View
import android.widget.Button

import net.mullvad.mullvadvpn.model.TunnelStateTransition

class SwitchLocationButton(val parentView: View) {
    private val button: Button = parentView.findViewById(R.id.switch_location)

    private var updateJob: Job? = null

    var state: TunnelStateTransition = TunnelStateTransition.Disconnected()
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
            when (state) {
                is TunnelStateTransition.Disconnected -> showLocation()
                is TunnelStateTransition.Disconnecting -> showLocation()
                is TunnelStateTransition.Connecting -> showLabel()
                is TunnelStateTransition.Connected -> showLabel()
                is TunnelStateTransition.Blocked -> showLocation()
            }
        }
    }

    private fun showLabel() {
        button.setText(R.string.switch_location)
    }

    private fun showLocation() {
        showLabel()
    }
}
