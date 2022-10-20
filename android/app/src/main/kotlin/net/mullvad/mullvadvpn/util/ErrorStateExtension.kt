package net.mullvad.mullvadvpn.util

import android.content.Context
import android.content.res.Resources
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.ui.extension.getAlwaysOnVpnAppName
import net.mullvad.talpid.tunnel.ErrorState
import net.mullvad.talpid.tunnel.ErrorStateCause
import net.mullvad.talpid.tunnel.ParameterGenerationError
import net.mullvad.talpid.util.addressString

fun ErrorState.getErrorNotificationResources(context: Context): ErrorNotificationMessage {
    return when {
        isBlocking -> ErrorNotificationMessage(
            R.string.blocking_all_connections,
            blockingErrorMessageId(this.cause)
        )
        cause is ErrorStateCause.InvalidDnsServers -> {
            ErrorNotificationMessage(
                R.string.blocking_all_connections,
                blockingErrorMessageId(this.cause),
                cause.addresses
                    .map { address -> address.addressString() }
                    .joinToString()
            )
        }
        cause is ErrorStateCause.VpnPermissionDenied -> {
            context.getAlwaysOnVpnAppName()
                ?.let {
                    ErrorNotificationMessage(
                        R.string.always_on_vpn_error_notification_title,
                        R.string.always_on_vpn_error_notification_content,
                        it
                    )
                } ?: ErrorNotificationMessage(
                R.string.vpn_permission_error_notification_title,
                R.string.vpn_permission_error_notification_message
            )
        }
        else -> ErrorNotificationMessage(
            R.string.critical_error,
            notBlockingErrorMessageId(this.cause)
        )
    }
}

data class ErrorNotificationMessage(
    val titleResourceId: Int,
    val messageResourceId: Int,
    val optionalMessageArgument: String? = null
) {
    fun getTitleText(resources: Resources): String {
        return resources.getString(titleResourceId)
    }

    fun getMessageText(resources: Resources): String {
        return if (optionalMessageArgument != null) {
            resources.getString(messageResourceId, optionalMessageArgument)
        } else {
            resources.getString(messageResourceId)
        }
    }
}

private fun blockingErrorMessageId(cause: ErrorStateCause): Int {
    return when (cause) {
        is ErrorStateCause.InvalidDnsServers -> R.string.invalid_dns_servers
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
}

private fun notBlockingErrorMessageId(cause: ErrorStateCause?): Int {
    return when (cause) {
        is ErrorStateCause.VpnPermissionDenied -> R.string.vpn_permission_denied_error
        else -> R.string.failed_to_block_internet
    }
}
