package net.mullvad.mullvadvpn.compose.component.notificationbanner

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.size
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import java.time.Duration
import net.mullvad.mullvadvpn.compose.component.MullvadTopBar
import net.mullvad.mullvadvpn.compose.util.isTv
import net.mullvad.mullvadvpn.lib.model.ErrorState
import net.mullvad.mullvadvpn.lib.model.ErrorStateCause
import net.mullvad.mullvadvpn.lib.model.InAppNotification
import net.mullvad.mullvadvpn.lib.model.VersionInfo
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.tv.NotificationBannerTv
import net.mullvad.mullvadvpn.lib.ui.component.AnimatedNotificationBanner

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
                    onClickShowAndroid16UpgradeInfo = {},
                    onClickDismissChangelog = {},
                    onClickDismissNewDevice = {},
                    onClickShowWireguardPortSettings = {},
                    onClickDismissAndroid16UpgradeWarning = {},
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
    contentFocusRequester: FocusRequester = FocusRequester(),
    openAppListing: () -> Unit,
    onClickShowAccount: () -> Unit,
    onClickShowChangelog: () -> Unit,
    onClickShowAndroid16UpgradeInfo: () -> Unit,
    onClickDismissChangelog: () -> Unit,
    onClickDismissNewDevice: () -> Unit,
    onClickShowWireguardPortSettings: () -> Unit,
    onClickDismissAndroid16UpgradeWarning: () -> Unit,
) {
    if (isTv()) {
        NotificationBannerTv(
            modifier = modifier,
            notification = notification,
            isPlayBuild = isPlayBuild,
            openAppListing = openAppListing,
            contentFocusRequester = contentFocusRequester,
            onClickShowAccount = onClickShowAccount,
            onClickShowChangelog = onClickShowChangelog,
            onClickShowAndroid16UpgradeInfo = onClickShowAndroid16UpgradeInfo,
            onClickDismissChangelog = onClickDismissChangelog,
            onClickDismissNewDevice = onClickDismissNewDevice,
            onClickShowWireguardPortSettings = onClickShowWireguardPortSettings,
            onClickDismissAndroid16UpgradeWarning = onClickDismissAndroid16UpgradeWarning,
        )
    } else {
        AnimatedNotificationBanner(
            modifier = modifier,
            notificationModifier = Modifier.fillMaxWidth(),
            notification = notification,
            isPlayBuild = isPlayBuild,
            openAppListing = openAppListing,
            contentFocusRequester = contentFocusRequester,
            onClickShowAccount = onClickShowAccount,
            onClickShowChangelog = onClickShowChangelog,
            onClickShowAndroid16UpgradeInfo = onClickShowAndroid16UpgradeInfo,
            onClickDismissChangelog = onClickDismissChangelog,
            onClickDismissNewDevice = onClickDismissNewDevice,
            onClickShowWireguardPortSettings = onClickShowWireguardPortSettings,
            onClickDismissAndroid16UpgradeWarning = onClickDismissAndroid16UpgradeWarning,
        )
    }
}
