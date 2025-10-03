package net.mullvad.mullvadvpn.usecase.inappnotification

import kotlinx.coroutines.flow.Flow
import net.mullvad.mullvadvpn.lib.model.InAppNotification

interface InAppNotificationUseCase {
    operator fun invoke(): Flow<InAppNotification?>
}
