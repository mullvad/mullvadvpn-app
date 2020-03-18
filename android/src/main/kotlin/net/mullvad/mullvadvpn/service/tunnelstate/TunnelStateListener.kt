package net.mullvad.mullvadvpn.service.tunnelstate

import android.content.Context
import android.content.SharedPreferences
import android.content.SharedPreferences.OnSharedPreferenceChangeListener
import net.mullvad.mullvadvpn.model.TunnelState

class TunnelStateListener(context: Context) {
    private val persistence = Persistence(context)

    private val listener = object : OnSharedPreferenceChangeListener {
        override fun onSharedPreferenceChanged(preferences: SharedPreferences, key: String) {
            state = persistence.state
        }
    }

    var state = persistence.state
        private set(value) {
            if (field != value) {
                field = value
                onStateChange?.invoke(value)
            }
        }

    var onStateChange: ((TunnelState) -> Unit)? = null
        set(value) {
            field = value

            if (value == null) {
                persistence.listener = null
            } else {
                persistence.listener = listener
                state = persistence.state
            }
        }
}
