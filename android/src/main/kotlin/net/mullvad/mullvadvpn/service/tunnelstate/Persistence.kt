package net.mullvad.mullvadvpn.service.tunnelstate

import android.content.Context
import android.content.SharedPreferences.OnSharedPreferenceChangeListener
import net.mullvad.mullvadvpn.model.TunnelState

private const val SHARED_PREFERENCES = "tunnel_state"
private const val KEY_TUNNEL_STATE = "tunnel_state"

internal class Persistence(context: Context) {
    val sharedPreferences =
        context.getSharedPreferences(SHARED_PREFERENCES, Context.MODE_PRIVATE)

    var state
        get() = loadState()
        set(value) {
            persistState(value)
        }

    var listener: OnSharedPreferenceChangeListener? = null
        set(value) {
            if (value != field) {
                if (field != null) {
                    sharedPreferences.unregisterOnSharedPreferenceChangeListener(field)
                }

                if (value != null) {
                    sharedPreferences.registerOnSharedPreferenceChangeListener(value)
                }

                field = value
            }
        }

    private fun loadState(): TunnelState {
        val description = sharedPreferences.getString(KEY_TUNNEL_STATE, TunnelState.DISCONNECTED)!!

        return TunnelState.fromString(description)
    }

    private fun persistState(state: TunnelState) {
        sharedPreferences
            .edit()
            .putString(KEY_TUNNEL_STATE, state.toString())
            .commit()
    }
}
