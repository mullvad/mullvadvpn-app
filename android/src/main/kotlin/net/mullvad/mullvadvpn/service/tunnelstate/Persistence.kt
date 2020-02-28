package net.mullvad.mullvadvpn.service.tunnelstate

import android.content.Context
import java.net.InetSocketAddress
import net.mullvad.mullvadvpn.model.TunnelState
import net.mullvad.talpid.net.Endpoint
import net.mullvad.talpid.net.TransportProtocol
import net.mullvad.talpid.net.TunnelEndpoint

private const val SHARED_PREFERENCES = "tunnel_state"
private const val KEY_TUNNEL_STATE = "tunnel_state"

// TODO: Maybe replace using this with actually persisting the endpoint information
private val dummyTunnelEndpoint = TunnelEndpoint(Endpoint(
    InetSocketAddress.createUnresolved("dummy", 53),
    TransportProtocol.Tcp
))

internal class Persistence(context: Context) {
    val sharedPreferences =
        context.getSharedPreferences(SHARED_PREFERENCES, Context.MODE_PRIVATE)

    var state
        get() = loadState()
        set(value) {
            persistState(value)
        }

    private fun loadState(): TunnelState {
        val description = sharedPreferences.getString(KEY_TUNNEL_STATE, TunnelState.DISCONNECTED)!!

        return TunnelState.fromString(description, dummyTunnelEndpoint)
    }

    private fun persistState(state: TunnelState) {
        sharedPreferences
            .edit()
            .putString(KEY_TUNNEL_STATE, state.toString())
            .commit()
    }
}
