package net.mullvad.mullvadvpn.lib.shared.compose

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.animateContentSize
import androidx.compose.animation.slideInVertically
import androidx.compose.animation.slideOutVertically
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.wrapContentWidth
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.text.toUpperCase
import androidx.constraintlayout.compose.ConstraintLayout
import androidx.constraintlayout.compose.Dimension
import net.mullvad.mullvadvpn.lib.shared.InAppNotification
import net.mullvad.mullvadvpn.lib.shared.StatusLevel
import net.mullvad.mullvadvpn.lib.shared.compose.test.NOTIFICATION_BANNER
import net.mullvad.mullvadvpn.lib.shared.compose.test.NOTIFICATION_BANNER_ACTION
import net.mullvad.mullvadvpn.lib.shared.compose.test.NOTIFICATION_BANNER_TEXT_ACTION
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.theme.color.warning

@Composable
fun AnimatedNotificationBanner(
    modifier: Modifier = Modifier,
    notificationModifier: Modifier = Modifier,
    notification: InAppNotification?,
    isPlayBuild: Boolean,
    openAppListing: () -> Unit,
    onClickShowAccount: () -> Unit,
    onClickShowChangelog: () -> Unit,
    onClickDismissChangelog: () -> Unit,
    onClickDismissNewDevice: () -> Unit,
) {
    // Fix for animating to invisible state
    val previous = rememberPrevious(current = notification, shouldUpdate = { _, _ -> true })
    AnimatedVisibility(
        modifier = modifier,
        visible = notification != null,
        enter = slideInVertically(initialOffsetY = { -it }),
        exit = slideOutVertically(targetOffsetY = { -it }),
    ) {
        val visibleNotification = notification ?: previous
        if (visibleNotification != null)
            Notification(
                modifier = notificationModifier,
                visibleNotification.toNotificationData(
                    isPlayBuild = isPlayBuild,
                    openAppListing,
                    onClickShowAccount,
                    onClickShowChangelog,
                    onClickDismissChangelog,
                    onClickDismissNewDevice,
                ),
            )
    }
}

@Composable
@Suppress("LongMethod")
private fun Notification(modifier: Modifier = Modifier, notificationBannerData: NotificationData) {
    val (title, message, statusLevel, action) = notificationBannerData
    ConstraintLayout(
        modifier =
            modifier
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
            text = title.toUpperCase(),
            modifier =
                Modifier.constrainAs(textTitle) {
                        top.linkTo(parent.top)
                        start.linkTo(status.end)
                        if (message != null) {
                            bottom.linkTo(textMessage.top)
                        } else {
                            bottom.linkTo(parent.bottom)
                        }
                        if (action != null) {
                            end.linkTo(actionIcon.start)
                        } else {
                            end.linkTo(parent.end)
                        }
                        width = Dimension.fillToConstraints
                    }
                    .padding(start = Dimens.smallPadding),
            style = MaterialTheme.typography.bodySmall,
            color = MaterialTheme.colorScheme.onSurface,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
        )
        message?.let { message ->
            Text(
                text = message.text,
                modifier =
                    Modifier.constrainAs(textMessage) {
                            top.linkTo(textTitle.bottom)
                            start.linkTo(textTitle.start)
                            if (action != null) {
                                end.linkTo(actionIcon.start)
                                bottom.linkTo(parent.bottom)
                            } else {
                                end.linkTo(parent.end)
                                bottom.linkTo(parent.bottom)
                            }
                            width = Dimension.fillToConstraints
                            height = Dimension.wrapContent
                        }
                        .padding(start = Dimens.smallPadding, top = Dimens.tinyPadding)
                        .wrapContentWidth(Alignment.Start)
                        .let {
                            if (message is NotificationMessage.ClickableText) {
                                it.clickable(
                                        onClickLabel = message.contentDescription,
                                        role = Role.Button,
                                    ) {
                                        message.onClick()
                                    }
                                    .testTag(NOTIFICATION_BANNER_TEXT_ACTION)
                            } else {
                                it
                            }
                        },
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
                            StatusLevel.Info -> MaterialTheme.colorScheme.tertiary
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

    IconButton(modifier = modifier.testTag(NOTIFICATION_BANNER_ACTION), onClick = onClick) {
        Icon(
            modifier = Modifier.padding(Dimens.notificationIconPadding),
            imageVector = imageVector,
            contentDescription = contentDescription,
            tint = MaterialTheme.colorScheme.onSurface,
        )
    }
}
