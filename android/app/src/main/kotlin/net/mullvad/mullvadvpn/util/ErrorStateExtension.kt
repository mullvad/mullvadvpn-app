package net.mullvad.mullvadvpn.util

import android.content.Context
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.ui.extension.getAlwaysOnVpnAppName
import net.mullvad.talpid.tunnel.ErrorState
import net.mullvad.talpid.tunnel.ErrorStateCause
import net.mullvad.talpid.util.addressString

fun ErrorState.getErrorNotificationResources(context: Context): ErrorNotificationMessage {
    return when {
        cause is ErrorStateCause.InvalidDnsServers -> {
            ErrorNotificationMessage(
                R.string.blocking_all_connections,
                cause.errorMessageId(),
                cause.addresses.joinToString { address -> address.addressString() }
            )
        }
        cause is ErrorStateCause.VpnPermissionDenied -> {
            resolveAlwaysOnVpnErrorNotificationMessage(context.getAlwaysOnVpnAppName())
        }
        isBlocking -> ErrorNotificationMessage(
            R.string.blocking_all_connections,
            cause.errorMessageId()
        )
        else -> ErrorNotificationMessage(
            R.string.critical_error,
            R.string.failed_to_block_internet
        )
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
