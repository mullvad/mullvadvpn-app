package net.mullvad.mullvadvpn.compose.component

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Column
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.tooling.preview.Preview
import java.time.Duration
import net.mullvad.mullvadvpn.compose.component.notificationbanner.Notification
import net.mullvad.mullvadvpn.compose.component.notificationbanner.toNotificationData
import net.mullvad.mullvadvpn.lib.model.ErrorState
import net.mullvad.mullvadvpn.lib.model.ErrorStateCause
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.repository.InAppNotification
import net.mullvad.mullvadvpn.ui.VersionInfo

class NotificationScreenshots {
    @Preview
    @Composable
    private fun PreviewNotificationBanner() {
        AppTheme {
            Column(Modifier.background(color = MaterialTheme.colorScheme.surface)) {
                val bannerDataList =
                    listOf(
                            InAppNotification.UnsupportedVersion(
                                versionInfo =
                                    VersionInfo(currentVersion = "1.0", isSupported = false)
                            ),
                            InAppNotification.AccountExpiry(expiry = Duration.ZERO),
                            InAppNotification.TunnelStateBlocked,
                            InAppNotification.NewDevice("Courageous Turtle"),
                            InAppNotification.TunnelStateError(
                                error =
                                    ErrorState(ErrorStateCause.FirewallPolicyError.Generic, true)
                            ),
                            InAppNotification.NewVersionChangelog,
                        )
                        .map { it.toNotificationData(false, {}, {}, {}, {}, {}) }

                bannerDataList.forEach { Notification(it) }
            }
        }
    }
}
