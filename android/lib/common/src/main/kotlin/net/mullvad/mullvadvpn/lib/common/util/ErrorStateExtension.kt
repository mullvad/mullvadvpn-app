package net.mullvad.mullvadvpn.lib.common.util

import android.content.Context
import net.mullvad.mullvadvpn.lib.common.R
import net.mullvad.talpid.tunnel.ErrorState
import net.mullvad.talpid.tunnel.ErrorStateCause
import net.mullvad.talpid.tunnel.ParameterGenerationError
import net.mullvad.talpid.util.addressString

fun ErrorState.getErrorNotificationResources(context: Context): ErrorNotificationMessage {
    return when {
        cause is ErrorStateCause.InvalidDnsServers -> {
            ErrorNotificationMessage(
                R.string.blocking_internet,
                cause.errorMessageId(),
                (cause as ErrorStateCause.InvalidDnsServers).addresses.joinToString { address ->
                    address.addressString()
                }
            )
        }
        cause is ErrorStateCause.VpnPermissionDenied -> {
            resolveAlwaysOnVpnErrorNotificationMessage(context.getAlwaysOnVpnAppName())
        }
        isBlocking -> ErrorNotificationMessage(R.string.blocking_internet, cause.errorMessageId())
        else -> ErrorNotificationMessage(R.string.critical_error, R.string.failed_to_block_internet)
    }
}

private fun resolveAlwaysOnVpnErrorNotificationMessage(
    alwaysOnVpnAppName: String?
): ErrorNotificationMessage {
    return if (alwaysOnVpnAppName != null) {
        ErrorNotificationMessage(
            R.string.always_on_vpn_error_notification_title,
            R.string.always_on_vpn_error_notification_content,
            alwaysOnVpnAppName
        )
    } else {
        ErrorNotificationMessage(
            R.string.vpn_permission_error_notification_title,
            R.string.vpn_permission_error_notification_message
        )
    }
}

fun ErrorStateCause.errorMessageId(): Int {
    return when (this) {
        is ErrorStateCause.InvalidDnsServers -> R.string.invalid_dns_servers
        is ErrorStateCause.AuthFailed -> R.string.auth_failed
        is ErrorStateCause.Ipv6Unavailable -> R.string.ipv6_unavailable
        is ErrorStateCause.SetFirewallPolicyError -> R.string.set_firewall_policy_error
        is ErrorStateCause.SetDnsError -> R.string.set_dns_error
        is ErrorStateCause.StartTunnelError -> R.string.start_tunnel_error
        is ErrorStateCause.IsOffline -> R.string.is_offline
        is ErrorStateCause.TunnelParameterError -> {
            when (error) {
                ParameterGenerationError.NoMatchingRelay,
                ParameterGenerationError.NoMatchingBridgeRelay -> {
                    R.string.no_matching_relay
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
