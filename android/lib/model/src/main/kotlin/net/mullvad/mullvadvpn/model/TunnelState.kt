package net.mullvad.mullvadvpn.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize
import net.mullvad.talpid.net.TunnelEndpoint
import net.mullvad.talpid.tunnel.ActionAfterDisconnect
import net.mullvad.talpid.tunnel.ErrorState
import net.mullvad.talpid.tunnel.ErrorStateCause
import net.mullvad.talpid.tunnel.FirewallPolicyError

sealed class TunnelState : Parcelable {
    @Parcelize object Disconnected : TunnelState(), Parcelable

    @Parcelize
    class Connecting(val endpoint: TunnelEndpoint?, val location: GeoIpLocation?) :
        TunnelState(), Parcelable

    @Parcelize
    class Connected(val endpoint: TunnelEndpoint, val location: GeoIpLocation?) :
        TunnelState(), Parcelable

    @Parcelize
    class Disconnecting(val actionAfterDisconnect: ActionAfterDisconnect) :
        TunnelState(), Parcelable

    @Parcelize class Error(val errorState: ErrorState) : TunnelState(), Parcelable

    fun isSecured(): Boolean {
        return when (this) {
            is Connected,
            is Connecting,
            is Disconnecting, -> true
            is Disconnected -> false
            is Error -> this.errorState.isBlocking
        }
    }

    override fun toString(): String =
        when (this) {
            is Disconnected -> DISCONNECTED
            is Connecting -> CONNECTING
            is Connected -> CONNECTED
            is Disconnecting -> {
                if (actionAfterDisconnect == ActionAfterDisconnect.Reconnect) {
                    RECONNECTING
                } else {
                    DISCONNECTING
                }
            }
            is Error -> {
                if (errorState.isBlocking) {
                    BLOCKING
                } else {
                    ERROR
                }
            }
        }

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
                DISCONNECTED -> Disconnected
                CONNECTING -> Connecting(endpoint, null)
                CONNECTED -> Connected(endpoint!!, null)
                RECONNECTING -> Disconnecting(ActionAfterDisconnect.Reconnect)
                DISCONNECTING -> Disconnecting(ActionAfterDisconnect.Nothing)
                BLOCKING -> Error(ErrorState(ErrorStateCause.StartTunnelError, true))
                ERROR -> {
                    Error(
                        ErrorState(
                            ErrorStateCause.SetFirewallPolicyError(FirewallPolicyError.Generic),
                            false
                        )
                    )
                }
                else ->
                    Error(
                        ErrorState(
                            ErrorStateCause.SetFirewallPolicyError(FirewallPolicyError.Generic),
                            false
                        )
                    )
            }
        }
    }
}
