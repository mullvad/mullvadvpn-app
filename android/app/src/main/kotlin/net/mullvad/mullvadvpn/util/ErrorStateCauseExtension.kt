package net.mullvad.mullvadvpn.util

import net.mullvad.mullvadvpn.R
import net.mullvad.talpid.tunnel.ErrorStateCause
import net.mullvad.talpid.tunnel.ParameterGenerationError

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
