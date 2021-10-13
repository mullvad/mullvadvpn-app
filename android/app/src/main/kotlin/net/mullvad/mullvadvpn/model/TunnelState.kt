package net.mullvad.mullvadvpn.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize
import net.mullvad.talpid.net.TunnelEndpoint
import net.mullvad.talpid.tunnel.ActionAfterDisconnect
import net.mullvad.talpid.tunnel.ErrorState
import net.mullvad.talpid.tunnel.ErrorStateCause

sealed class TunnelState() : Parcelable {
    @Parcelize
    object Disconnected : TunnelState(), Parcelable

    @Parcelize
    class Connecting(
        val endpoint: TunnelEndpoint?,
        val location: GeoIpLocation?
    ) : TunnelState(), Parcelable

    @Parcelize
    class Connected(
        val endpoint: TunnelEndpoint,
        val location: GeoIpLocation?
    ) : TunnelState(), Parcelable

    @Parcelize
    class Disconnecting(
        val actionAfterDisconnect: ActionAfterDisconnect
    ) : TunnelState(), Parcelable

    @Parcelize
    class Error(val errorState: ErrorState) : TunnelState(), Parcelable

    companion object {
        const val DISCONNECTED = "disconnected"
        const val CONNECTING = "connecting"
        const val CONNECTED = "connected"
        const val RECONNECTING = "reconnecting"
        const val DISCONNECTING = "disconnecting"
        const val BLOCKING = "blocking"
        const val ERROR = "error"

        fun fromString(description: String, endpoint: TunnelEndpoint?): TunnelState {
            return when (description) {
                DISCONNECTED -> TunnelState.Disconnected
                CONNECTING -> TunnelState.Connecting(endpoint, null)
                CONNECTED -> TunnelState.Connected(endpoint!!, null)
                RECONNECTING -> TunnelState.Disconnecting(ActionAfterDisconnect.Reconnect)
                DISCONNECTING -> TunnelState.Disconnecting(ActionAfterDisconnect.Nothing)
                BLOCKING -> TunnelState.Error(ErrorState(ErrorStateCause.StartTunnelError, true))
                ERROR -> {
                    TunnelState.Error(ErrorState(ErrorStateCause.SetFirewallPolicyError, false))
                }
                else -> TunnelState.Error(ErrorState(ErrorStateCause.SetFirewallPolicyError, false))
            }
        }
    }

    override fun toString(): String = when (this) {
        is TunnelState.Disconnected -> DISCONNECTED
        is TunnelState.Connecting -> CONNECTING
        is TunnelState.Connected -> CONNECTED
        is TunnelState.Disconnecting -> {
            if (actionAfterDisconnect == ActionAfterDisconnect.Reconnect) {
                RECONNECTING
            } else {
                DISCONNECTING
            }
        }
        is TunnelState.Error -> {
            if (errorState.isBlocking) {
                BLOCKING
            } else {
                ERROR
            }
        }
    }
}
