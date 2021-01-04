package net.mullvad.mullvadvpn.ui.notification

import android.content.Context
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.model.TunnelState
import net.mullvad.mullvadvpn.service.endpoint.ConnectionProxy
import net.mullvad.talpid.tunnel.ActionAfterDisconnect
import net.mullvad.talpid.tunnel.ErrorState
import net.mullvad.talpid.tunnel.ErrorStateCause
import net.mullvad.talpid.tunnel.ParameterGenerationError
import net.mullvad.talpid.util.addressString

class TunnelStateNotification(
    private val context: Context,
    private val connectionProxy: ConnectionProxy
) : InAppNotification() {
    private val blockingTitle = context.getString(R.string.blocking_internet)
    private val notBlockingTitle = context.getString(R.string.not_blocking_internet)

    init {
        status = StatusLevel.Error
        onClick = null
        showIcon = false
    }

    override fun onResume() {
        connectionProxy.onStateChange.subscribe(this) { tunnelState ->
            jobTracker.newUiJob("updateTunnelState") {
                updateTunnelState(tunnelState)
            }
        }
    }

    override fun onPause() {
        connectionProxy.onStateChange.unsubscribe(this)
    }

    private fun updateTunnelState(state: TunnelState) {
        when (state) {
            is TunnelState.Disconnecting -> {
                when (state.actionAfterDisconnect) {
                    ActionAfterDisconnect.Nothing -> hide()
                    ActionAfterDisconnect.Block -> show(null)
                    ActionAfterDisconnect.Reconnect -> show(null)
                }
            }
            is TunnelState.Disconnected -> hide()
            is TunnelState.Connecting -> show(null)
            is TunnelState.Connected -> hide()
            is TunnelState.Error -> show(state.errorState)
        }

        update()
    }

    private fun show(error: ErrorState?) {
        // if the error state is null, we can assume that we are secure
        if (error?.isBlocking ?: true) {
            title = blockingTitle
            message = error?.cause?.let { cause -> blockingErrorMessage(cause) }
        } else {
            title = notBlockingTitle
            message = notBlockingErrorMessage(error?.cause)
        }

        shouldShow = true
    }

    private fun blockingErrorMessage(cause: ErrorStateCause): String {
        val messageId = when (cause) {
            is ErrorStateCause.InvalidDnsServers -> {
                val addresses = cause.addresses
                    .map { address -> address.addressString() }
                    .joinToString()

                return context.getString(R.string.invalid_dns_servers, addresses)
            }
            is ErrorStateCause.AuthFailed -> R.string.auth_failed
            is ErrorStateCause.Ipv6Unavailable -> R.string.ipv6_unavailable
            is ErrorStateCause.SetFirewallPolicyError -> R.string.set_firewall_policy_error
            is ErrorStateCause.SetDnsError -> R.string.set_dns_error
            is ErrorStateCause.StartTunnelError -> R.string.start_tunnel_error
            is ErrorStateCause.IsOffline -> R.string.is_offline
            is ErrorStateCause.TunnelParameterError -> {
                when (cause.error) {
                    ParameterGenerationError.NoMatchingRelay -> R.string.no_matching_relay
                    ParameterGenerationError.NoMatchingBridgeRelay -> {
                        R.string.no_matching_bridge_relay
                    }
                    ParameterGenerationError.NoWireguardKey -> R.string.no_wireguard_key
                    ParameterGenerationError.CustomTunnelHostResultionError -> {
                        R.string.custom_tunnel_host_resolution_error
                    }
                }
            }
            is ErrorStateCause.VpnPermissionDenied -> R.string.vpn_permission_denied_error
        }

        return context.getString(messageId)
    }

    private fun notBlockingErrorMessage(cause: ErrorStateCause?): String {
        val messageId = when (cause) {
            is ErrorStateCause.VpnPermissionDenied -> R.string.vpn_permission_denied_error
            else -> R.string.failed_to_block_internet
        }

        return context.getString(messageId)
    }

    private fun hide() {
        shouldShow = false
    }
}
