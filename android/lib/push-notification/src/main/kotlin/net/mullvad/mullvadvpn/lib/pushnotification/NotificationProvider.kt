package net.mullvad.mullvadvpn.lib.pushnotification

import kotlinx.coroutines.flow.Flow
import net.mullvad.mullvadvpn.lib.model.NotificationUpdate

interface NotificationProvider<D> {
    val notifications: Flow<NotificationUpdate<D>>
}
