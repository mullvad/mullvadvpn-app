package net.mullvad.mullvadvpn.lib.tv

import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import net.mullvad.mullvadvpn.lib.model.InAppNotification
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.ui.component.AnimatedNotificationBanner

@Preview
@Composable
fun PreviewNotificationBannerTv() {
    AppTheme {
        NotificationBannerTv(
            notification = InAppNotification.NewDevice("Sad Panda"),
            isPlayBuild = true,
            openAppListing = {},
            onClickShowAccount = {},
            onClickShowChangelog = {},
            onClickShowAndroid16UpgradeInfo = {},
            onClickDismissChangelog = {},
            onClickDismissNewDevice = {},
            onClickShowWireguardPortSettings = {},
            onClickDismissAndroid16UpgradeWarning = {},
        )
    }
}

@Composable
fun NotificationBannerTv(
    modifier: Modifier = Modifier,
    notification: InAppNotification?,
    isPlayBuild: Boolean,
    openAppListing: () -> Unit,
    contentFocusRequester: FocusRequester = FocusRequester(),
    onClickShowAccount: () -> Unit,
    onClickShowChangelog: () -> Unit,
    onClickShowAndroid16UpgradeInfo: () -> Unit,
    onClickDismissChangelog: () -> Unit,
    onClickDismissNewDevice: () -> Unit,
    onClickShowWireguardPortSettings: () -> Unit,
    onClickDismissAndroid16UpgradeWarning: () -> Unit,
) {
    AnimatedNotificationBanner(
        modifier = modifier,
        notificationModifier =
            Modifier.width(Dimens.connectionCardMaxWidth)
                .padding(start = Dimens.mediumPadding, end = Dimens.mediumPadding)
                .clip(
                    RoundedCornerShape(
                        bottomEnd = Dimens.mediumPadding,
                        bottomStart = Dimens.mediumPadding,
                        topStart = 0.dp,
                        topEnd = 0.dp,
                    )
                ),
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
