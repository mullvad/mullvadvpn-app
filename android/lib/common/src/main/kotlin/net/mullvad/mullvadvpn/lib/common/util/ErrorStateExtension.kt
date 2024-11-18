package net.mullvad.mullvadvpn.lib.common.util

import java.net.InetAddress
import net.mullvad.mullvadvpn.lib.common.R
import net.mullvad.mullvadvpn.lib.model.AuthFailedError
import net.mullvad.mullvadvpn.lib.model.ErrorState
import net.mullvad.mullvadvpn.lib.model.ErrorStateCause
import net.mullvad.mullvadvpn.lib.model.ParameterGenerationError
import net.mullvad.mullvadvpn.lib.model.PrepareError

fun ErrorState.notificationResources(): ErrorNotificationMessage {
    val cause = this.cause
    return when {
        cause is ErrorStateCause.InvalidDnsServers -> {
            ErrorNotificationMessage(
                R.string.blocking_internet,
                cause.errorMessageId(),
                cause.addresses.joinToString { address -> address.addressString() },
            )
        }
        cause is ErrorStateCause.VpnPermissionDenied ->
            cause.prepareError.errorNotificationMessage()

        isBlocking -> ErrorNotificationMessage(R.string.blocking_internet, cause.errorMessageId())
        else -> ErrorNotificationMessage(R.string.critical_error, R.string.failed_to_block_internet)
    }
}

private fun PrepareError.errorNotificationMessage(): ErrorNotificationMessage =
    when (this) {
        is PrepareError.NotPrepared ->
            ErrorNotificationMessage(
                R.string.vpn_permission_error_notification_title,
                R.string.vpn_permission_error_notification_message,
            )
        is PrepareError.OtherAlwaysOnApp ->
            ErrorNotificationMessage(
                R.string.legacy_always_on_vpn_error_notification_title,
                R.string.always_on_vpn_error_notification_content,
                appName,
            )
        is PrepareError.LegacyLockdown ->
            ErrorNotificationMessage(
                R.string.legacy_always_on_vpn_error_notification_title,
                R.string.legacy_always_on_vpn_error_notification_content,
            )
    }

fun ErrorStateCause.errorMessageId(): Int =
    when (this) {
        is ErrorStateCause.InvalidDnsServers -> R.string.invalid_dns_servers
        is ErrorStateCause.AuthFailed -> error.errorMessageId()
        is ErrorStateCause.Ipv6Unavailable -> R.string.ipv6_unavailable
        is ErrorStateCause.FirewallPolicyError -> R.string.set_firewall_policy_error
        is ErrorStateCause.DnsError -> R.string.set_dns_error
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

fun AuthFailedError.errorMessageId(): Int =
    when (this) {
        AuthFailedError.ExpiredAccount -> R.string.account_credit_has_expired
        AuthFailedError.InvalidAccount,
        AuthFailedError.TooManyConnections,
        AuthFailedError.Unknown -> R.string.auth_failed
    }

fun InetAddress.addressString(): String {
    val hostNameAndAddress = this.toString().split('/', limit = 2)
    val address = hostNameAndAddress[1]

    return address
}
