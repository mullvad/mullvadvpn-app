package net.mullvad.mullvadvpn.lib.tv

import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.unit.dp
import net.mullvad.mullvadvpn.lib.shared.compose.AnimatedNotificationBanner
import net.mullvad.mullvadvpn.lib.theme.Dimens

@Composable
fun NotificationBannerTv(
    modifier: Modifier = Modifier,
    notification: net.mullvad.mullvadvpn.lib.shared.InAppNotification?,
    isPlayBuild: Boolean,
    openAppListing: () -> Unit,
    onClickShowAccount: () -> Unit,
    onClickShowChangelog: () -> Unit,
    onClickDismissChangelog: () -> Unit,
    onClickDismissNewDevice: () -> Unit,
) {
    AnimatedNotificationBanner(
        modifier = modifier,
        notificationModifier =
            Modifier.width(Dimens.connectionCardMaxWidth)
                .padding(start = Dimens.mediumPadding, end = Dimens.mediumPadding)
                .clip(
                    RoundedCornerShape(
                        bottomEnd = 16.dp,
                        bottomStart = 16.dp,
                        topStart = 0.dp,
                        topEnd = 0.dp,
                    )
                ),
        notification = notification,
        isPlayBuild = isPlayBuild,
        openAppListing = openAppListing,
        onClickShowAccount = onClickShowAccount,
        onClickShowChangelog = onClickShowChangelog,
        onClickDismissChangelog = onClickDismissChangelog,
        onClickDismissNewDevice = onClickDismissNewDevice,
    )
}
