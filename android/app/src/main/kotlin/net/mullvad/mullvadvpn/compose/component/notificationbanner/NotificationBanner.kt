package net.mullvad.mullvadvpn.compose.component.notificationbanner

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.animateContentSize
import androidx.compose.animation.slideInVertically
import androidx.compose.animation.slideOutVertically
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.constraintlayout.compose.ConstraintLayout
import androidx.constraintlayout.compose.Dimension
import net.mullvad.mullvadvpn.compose.component.MullvadTopBar
import net.mullvad.mullvadvpn.compose.test.NOTIFICATION_BANNER
import net.mullvad.mullvadvpn.compose.test.NOTIFICATION_BANNER_ACTION
import net.mullvad.mullvadvpn.compose.util.rememberPrevious
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.theme.color.AlphaDescription
import net.mullvad.mullvadvpn.repository.InAppNotification
import net.mullvad.mullvadvpn.ui.VersionInfo
import net.mullvad.mullvadvpn.ui.notification.StatusLevel
import net.mullvad.talpid.tunnel.ErrorState
import net.mullvad.talpid.tunnel.ErrorStateCause
import net.mullvad.talpid.tunnel.FirewallPolicyError
import org.joda.time.DateTime

@Preview
@Composable
private fun PreviewNotificationBanner() {
    AppTheme {
        Column(
            Modifier.background(color = MaterialTheme.colorScheme.surface),
        ) {
            val bannerDataList =
                listOf(
                        InAppNotification.UnsupportedVersion(
                            versionInfo =
                                VersionInfo(
                                    currentVersion = null,
                                    upgradeVersion = null,
                                    isOutdated = true,
                                    isSupported = false
                                ),
                        ),
                        InAppNotification.AccountExpiry(expiry = DateTime.now()),
                        InAppNotification.TunnelStateBlocked,
                        InAppNotification.NewDevice("Courageous Turtle"),
                        InAppNotification.TunnelStateError(
                            error =
                                ErrorState(
                                    ErrorStateCause.SetFirewallPolicyError(
                                        FirewallPolicyError.Generic
                                    ),
                                    true
                                )
                        )
                    )
                    .map { it.toNotificationData({}, {}, {}) }

            bannerDataList.forEach {
                MullvadTopBar(
                    containerColor = MaterialTheme.colorScheme.primary,
                    onSettingsClicked = {},
                    onAccountClicked = {},
                    iconTintColor = MaterialTheme.colorScheme.primary
                )
                Notification(it)
                Spacer(modifier = Modifier.size(16.dp))
            }
        }
    }
}

@Composable
fun NotificationBanner(
    notification: InAppNotification?,
    onClickUpdateVersion: () -> Unit,
    onClickShowAccount: () -> Unit,
    onClickDismissNewDevice: () -> Unit
) {
    // Fix for animating to invisible state
    val previous = rememberPrevious(current = notification, shouldUpdate = { _, _ -> true })
    AnimatedVisibility(
        visible = notification != null,
        enter = slideInVertically(initialOffsetY = { -it }),
        exit = slideOutVertically(targetOffsetY = { -it }),
        modifier = Modifier.animateContentSize()
    ) {
        val visibleNotification = notification ?: previous
        if (visibleNotification != null)
            Notification(
                visibleNotification.toNotificationData(
                    onClickUpdateVersion,
                    onClickShowAccount,
                    onClickDismissNewDevice
                )
            )
    }
}

@Composable
private fun Notification(notificationBannerData: NotificationData) {
    val (title, message, statusLevel, action) = notificationBannerData
    ConstraintLayout(
        modifier =
            Modifier.fillMaxWidth()
                .background(color = MaterialTheme.colorScheme.background)
                .padding(
                    start = Dimens.notificationBannerStartPadding,
                    end = Dimens.notificationBannerEndPadding,
                    top = Dimens.smallPadding,
                    bottom = Dimens.smallPadding
                )
                .animateContentSize()
                .testTag(NOTIFICATION_BANNER)
    ) {
        val (status, textTitle, textMessage, actionIcon) = createRefs()
        Box(
            modifier =
                Modifier.background(
                        color =
                            when (statusLevel) {
                                StatusLevel.Error -> MaterialTheme.colorScheme.error
                                StatusLevel.Warning -> MaterialTheme.colorScheme.errorContainer
                                StatusLevel.Info -> MaterialTheme.colorScheme.surface
                            },
                        shape = CircleShape
                    )
                    .size(Dimens.notificationStatusIconSize)
                    .constrainAs(status) {
                        top.linkTo(textTitle.top)
                        start.linkTo(parent.start)
                        bottom.linkTo(textTitle.bottom)
                    }
        )
        Text(
            text = title.uppercase(),
            modifier =
                Modifier.constrainAs(textTitle) {
                        top.linkTo(parent.top)
                        start.linkTo(status.end)
                        bottom.linkTo(anchor = textMessage.top)
                        end.linkTo(actionIcon.start)
                        width = Dimension.fillToConstraints
                    }
                    .padding(start = Dimens.smallPadding),
            style = MaterialTheme.typography.bodySmall,
            color = MaterialTheme.colorScheme.onBackground,
        )
        message?.let {
            Text(
                text = message,
                modifier =
                    Modifier.constrainAs(textMessage) {
                            top.linkTo(textTitle.bottom)
                            start.linkTo(textTitle.start)
                            bottom.linkTo(parent.bottom)
                            if (action != null) {
                                end.linkTo(actionIcon.start)
                            } else {
                                end.linkTo(parent.end)
                            }
                            width = Dimension.fillToConstraints
                        }
                        .padding(start = Dimens.smallPadding),
                color = MaterialTheme.colorScheme.onBackground.copy(alpha = AlphaDescription),
                style = MaterialTheme.typography.labelMedium
            )
        }
        action?.let {
            IconButton(
                modifier =
                    Modifier.constrainAs(actionIcon) {
                            top.linkTo(parent.top)
                            end.linkTo(parent.end)
                            bottom.linkTo(parent.bottom)
                        }
                        .testTag(NOTIFICATION_BANNER_ACTION)
                        .padding(all = Dimens.notificationEndIconPadding),
                onClick = it.onClick
            ) {
                Icon(
                    painter = painterResource(id = it.icon),
                    contentDescription = null,
                    tint = Color.Unspecified
                )
            }
        }
    }
}
