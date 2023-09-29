package net.mullvad.mullvadvpn.service.notifications

import net.mullvad.mullvadvpn.lib.common.constant.KEY_CONNECT_ACTION
import net.mullvad.mullvadvpn.lib.common.constant.KEY_DISCONNECT_ACTION
import net.mullvad.mullvadvpn.model.TunnelState
import net.mullvad.mullvadvpn.service.R
import net.mullvad.talpid.tunnel.ActionAfterDisconnect

enum class TunnelStateNotificationAction {
    Connect,
    Disconnect,
    Cancel,
    Dismiss;

    val text
        get() =
            when (this) {
                Connect -> R.string.connect
                Disconnect -> R.string.disconnect
                Cancel -> R.string.cancel
                Dismiss -> R.string.dismiss
            }

    val key
        get() =
            when (this) {
                Connect -> KEY_CONNECT_ACTION
                else -> KEY_DISCONNECT_ACTION
            }

    val icon
        get() =
            when (this) {
                Connect -> R.drawable.icon_notification_connect
                else -> R.drawable.icon_notification_disconnect
            }

    companion object {
        fun from(tunnelState: TunnelState) =
            when (tunnelState) {
                is TunnelState.Disconnected -> Connect
                is TunnelState.Connecting -> Cancel
                is TunnelState.Connected -> Disconnect
                is TunnelState.Disconnecting -> {
                    when (tunnelState.actionAfterDisconnect) {
                        ActionAfterDisconnect.Reconnect -> Cancel
                        else -> Connect
                    }
                }
                is TunnelState.Error -> {
                    if (tunnelState.errorState.isBlocking) {
                        Disconnect
                    } else {
                        Dismiss
                    }
                }
            }
    }
}
