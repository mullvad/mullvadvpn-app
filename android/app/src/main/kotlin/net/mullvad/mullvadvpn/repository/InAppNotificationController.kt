package net.mullvad.mullvadvpn.repository

import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import net.mullvad.mullvadvpn.usecase.AccountExpiryInAppNotificationUseCase
import net.mullvad.mullvadvpn.usecase.NewChangelogNotificationUseCase
import net.mullvad.mullvadvpn.usecase.NewDeviceNotificationUseCase
import net.mullvad.mullvadvpn.usecase.TunnelStateNotificationUseCase
import net.mullvad.mullvadvpn.usecase.VersionNotificationUseCase

class InAppNotificationController(
    accountExpiryInAppNotificationUseCase: AccountExpiryInAppNotificationUseCase,
    newDeviceNotificationUseCase: NewDeviceNotificationUseCase,
    newChangelogNotificationUseCase: NewChangelogNotificationUseCase,
    versionNotificationUseCase: VersionNotificationUseCase,
    tunnelStateNotificationUseCase: TunnelStateNotificationUseCase,
    scope: CoroutineScope,
) {

    val notifications =
        combine(
                tunnelStateNotificationUseCase(),
                versionNotificationUseCase(),
                accountExpiryInAppNotificationUseCase(),
                newDeviceNotificationUseCase(),
                newChangelogNotificationUseCase(),
            ) { a, b, c, d, e ->
                a + b + c + d + e
            }
            .map {
                it.sortedWith(
                    compareBy(
                        { notification -> notification.statusLevel.ordinal },
                        { notification -> -notification.priority },
                    )
                )
            }
            .stateIn(scope, SharingStarted.Eagerly, emptyList())
}
