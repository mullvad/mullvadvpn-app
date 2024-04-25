package net.mullvad.mullvadvpn.service.notifications

import kotlinx.coroutines.flow.Flow
import net.mullvad.mullvadvpn.model.Notification

interface NotificationProvider {
    val notifications: Flow<Notification>
}

// class ForegroundNotificationManager(
//    shouldBeOnForegroundProvider: ShouldBeOnForegroundProvider,
//    vpnService: MullvadVpnService,
//    tunnelStateNotificationProvider: TunnelStateNotificationProvider
// ) {}
