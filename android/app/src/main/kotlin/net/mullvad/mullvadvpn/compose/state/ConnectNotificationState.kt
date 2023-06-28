package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.ui.VersionInfo
import net.mullvad.talpid.tunnel.ErrorState
import org.joda.time.DateTime

sealed interface ConnectNotificationState {
    data object ShowTunnelStateNotificationBlocked : ConnectNotificationState

    data class ShowTunnelStateNotificationError(val error: ErrorState) : ConnectNotificationState

    data class ShowVersionInfoNotification(val versionInfo: VersionInfo) : ConnectNotificationState

    data class ShowAccountExpiryNotification(val expiry: DateTime) : ConnectNotificationState

    data object HideNotification : ConnectNotificationState
}
