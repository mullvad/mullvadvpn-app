package net.mullvad.mullvadvpn.service.notifications

import kotlinx.coroutines.flow.Flow
import net.mullvad.mullvadvpn.model.NotificationUpdate

interface NotificationProvider<D> {
    val notifications: Flow<NotificationUpdate<D>>
}
