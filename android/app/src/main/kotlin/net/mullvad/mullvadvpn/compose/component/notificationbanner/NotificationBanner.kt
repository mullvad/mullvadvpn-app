package net.mullvad.mullvadvpn.compose.component.notificationbanner

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.size
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import java.time.Duration
import net.mullvad.mullvadvpn.compose.component.MullvadTopBar
import net.mullvad.mullvadvpn.compose.util.isTv
import net.mullvad.mullvadvpn.lib.model.ErrorState
import net.mullvad.mullvadvpn.lib.model.ErrorStateCause
import net.mullvad.mullvadvpn.lib.shared.InAppNotification
import net.mullvad.mullvadvpn.lib.shared.VersionInfo
import net.mullvad.mullvadvpn.lib.shared.compose.AnimatedNotificationBanner
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.tv.NotificationBannerTv

@Preview
@Composable
private fun PreviewNotificationBanner() {
    AppTheme {
        Column(Modifier.background(color = MaterialTheme.colorScheme.surface)) {
            val bannerDataList =
                listOf(
                    InAppNotification.UnsupportedVersion(
                        versionInfo = VersionInfo(currentVersion = "1.0", isSupported = false)
                    ),
                    InAppNotification.AccountExpiry(expiry = Duration.ZERO),
                    InAppNotification.TunnelStateBlocked,
                    InAppNotification.NewDevice("Courageous Turtle"),
                    InAppNotification.TunnelStateError(
                        error = ErrorState(ErrorStateCause.FirewallPolicyError.Generic, true)
                    ),
                    InAppNotification.NewVersionChangelog,
                )

            bannerDataList.forEach {
                MullvadTopBar(
                    containerColor = MaterialTheme.colorScheme.primary,
                    onSettingsClicked = {},
                    onAccountClicked = {},
                    iconTintColor = MaterialTheme.colorScheme.primary,
                )
                NotificationBanner(
                    notification = it,
                    isPlayBuild = false,
                    openAppListing = {},
                    onClickShowAccount = {},
                    onClickShowChangelog = {},
                    onClickDismissChangelog = {},
                    onClickDismissNewDevice = {},
                )
                Spacer(modifier = Modifier.size(16.dp))
            }
        }
    }
}

@Composable
fun NotificationBanner(
    modifier: Modifier = Modifier,
    notification: InAppNotification?,
    isPlayBuild: Boolean,
    openAppListing: () -> Unit,
    onClickShowAccount: () -> Unit,
    onClickShowChangelog: () -> Unit,
    onClickDismissChangelog: () -> Unit,
    onClickDismissNewDevice: () -> Unit,
) {
    if (isTv()) {
        NotificationBannerTv(
            modifier = modifier,
            notification = notification,
            isPlayBuild = isPlayBuild,
            openAppListing = openAppListing,
            onClickShowAccount = onClickShowAccount,
            onClickShowChangelog = onClickShowChangelog,
            onClickDismissChangelog = onClickDismissChangelog,
            onClickDismissNewDevice = onClickDismissNewDevice,
        )
    } else {
        AnimatedNotificationBanner(
            modifier = modifier,
            notificationModifier = Modifier.fillMaxWidth(),
            notification = notification,
            isPlayBuild = isPlayBuild,
            openAppListing = openAppListing,
            onClickShowAccount = onClickShowAccount,
            onClickShowChangelog = onClickShowChangelog,
            onClickDismissChangelog = onClickDismissChangelog,
            onClickDismissNewDevice = onClickDismissNewDevice,
        )
    }
}
