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
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.constraintlayout.compose.ConstraintLayout
import androidx.constraintlayout.compose.Dimension
import net.mullvad.mullvadvpn.compose.component.MullvadTopBar
import net.mullvad.mullvadvpn.compose.test.NOTIFICATION_BANNER
import net.mullvad.mullvadvpn.compose.test.NOTIFICATION_BANNER_ACTION
import net.mullvad.mullvadvpn.compose.util.rememberPrevious
import net.mullvad.mullvadvpn.lib.model.ErrorState
import net.mullvad.mullvadvpn.lib.model.ErrorStateCause
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.theme.color.warning
import net.mullvad.mullvadvpn.repository.InAppNotification
import net.mullvad.mullvadvpn.ui.VersionInfo
import net.mullvad.mullvadvpn.ui.notification.StatusLevel
import org.joda.time.Duration

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
                    )
                    .map { it.toNotificationData(false, {}, {}, {}) }

            bannerDataList.forEach {
                MullvadTopBar(
                    containerColor = MaterialTheme.colorScheme.primary,
                    onSettingsClicked = {},
                    onAccountClicked = {},
                    iconTintColor = MaterialTheme.colorScheme.primary,
                )
                Notification(it)
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
    onClickUpdateVersion: () -> Unit,
    onClickShowAccount: () -> Unit,
    onClickDismissNewDevice: () -> Unit,
) {
    // Fix for animating to invisible state
    val previous = rememberPrevious(current = notification, shouldUpdate = { _, _ -> true })
    AnimatedVisibility(
        visible = notification != null,
        enter = slideInVertically(initialOffsetY = { -it }),
        exit = slideOutVertically(targetOffsetY = { -it }),
        modifier = modifier,
    ) {
        val visibleNotification = notification ?: previous
        if (visibleNotification != null)
            Notification(
                visibleNotification.toNotificationData(
                    isPlayBuild = isPlayBuild,
                    onClickUpdateVersion,
                    onClickShowAccount,
                    onClickDismissNewDevice,
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
                .background(color = MaterialTheme.colorScheme.surfaceContainer)
                .padding(
                    start = Dimens.notificationBannerStartPadding,
                    end = Dimens.notificationBannerEndPadding,
                    top = Dimens.smallPadding,
                    bottom = Dimens.smallPadding,
                )
                .animateContentSize()
                .testTag(NOTIFICATION_BANNER)
    ) {
        val (status, textTitle, textMessage, actionIcon) = createRefs()
        NotificationDot(
            statusLevel,
            Modifier.constrainAs(status) {
                top.linkTo(textTitle.top)
                start.linkTo(parent.start)
                bottom.linkTo(textTitle.bottom)
            },
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
            color = MaterialTheme.colorScheme.onSurface,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
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
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                style = MaterialTheme.typography.labelMedium,
            )
        }
        action?.let {
            NotificationAction(
                it.icon,
                onClick = it.onClick,
                contentDescription = it.contentDescription,
                modifier =
                    Modifier.constrainAs(actionIcon) {
                        top.linkTo(parent.top)
                        end.linkTo(parent.end)
                        bottom.linkTo(parent.bottom)
                    },
            )
        }
    }
}

@Composable
private fun NotificationDot(statusLevel: StatusLevel, modifier: Modifier) {
    Box(
        modifier =
            modifier
                .background(
                    color =
                        when (statusLevel) {
                            StatusLevel.Error -> MaterialTheme.colorScheme.error
                            StatusLevel.Warning -> MaterialTheme.colorScheme.warning
                            StatusLevel.Info -> MaterialTheme.colorScheme.surfaceContainer
                        },
                    shape = CircleShape,
                )
                .size(Dimens.notificationStatusIconSize)
    )
}

@Composable
private fun NotificationAction(
    imageVector: ImageVector,
    contentDescription: String?,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
) {
    IconButton(
        modifier =
            modifier
                .testTag(NOTIFICATION_BANNER_ACTION)
                .padding(all = Dimens.notificationEndIconPadding),
        onClick = onClick,
    ) {
        Icon(
            modifier = Modifier.padding(Dimens.notificationIconPadding),
            imageVector = imageVector,
            contentDescription = contentDescription,
            tint = MaterialTheme.colorScheme.onSurface,
        )
    }
}
