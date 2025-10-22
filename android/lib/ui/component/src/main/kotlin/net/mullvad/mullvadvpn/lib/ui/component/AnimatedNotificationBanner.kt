package net.mullvad.mullvadvpn.lib.ui.component

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
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.text.toUpperCase
import androidx.constraintlayout.compose.ConstraintLayout
import androidx.constraintlayout.compose.Dimension
import net.mullvad.mullvadvpn.lib.model.InAppNotification
import net.mullvad.mullvadvpn.lib.model.StatusLevel
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.theme.color.warning
import net.mullvad.mullvadvpn.lib.ui.tag.NOTIFICATION_BANNER_ACTION_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.NOTIFICATION_BANNER_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.NOTIFICATION_BANNER_TEXT_ACTION_TEST_TAG

@Composable
fun AnimatedNotificationBanner(
    modifier: Modifier = Modifier,
    notificationModifier: Modifier = Modifier,
    notification: InAppNotification?,
    isPlayBuild: Boolean,
    openAppListing: () -> Unit,
    contentFocusRequester: FocusRequester,
    onClickShowAccount: () -> Unit,
    onClickShowChangelog: () -> Unit,
    onClickShowAndroid16UpgradeInfo: () -> Unit,
    onClickDismissChangelog: () -> Unit,
    onClickDismissNewDevice: () -> Unit,
    onClickShowWireguardPortSettings: () -> Unit,
    onClickDismissAndroid16UpgradeWarning: () -> Unit,
) {
    // Fix for animating to invisible state
    val previous = rememberPrevious(current = notification, shouldUpdate = { _, _ -> true })

    val isVisible = notification != null

    val isNotificationDismissed = !isVisible && previous != null
    val notificationHasFocus = remember { mutableStateOf(false) }
    LaunchedEffect(isNotificationDismissed) {
        // If the notification is dismissed, we want to reset the previous notification
        if (isNotificationDismissed && notificationHasFocus.value) {
            contentFocusRequester.requestFocus()
        }
    }
    AnimatedVisibility(
        modifier = modifier.onFocusChanged { notificationHasFocus.value = it.hasFocus },
        visible = isVisible,
        enter = slideInVertically(initialOffsetY = { -it }),
        exit = slideOutVertically(targetOffsetY = { -it }),
    ) {
        val visibleNotification = notification ?: previous
        if (visibleNotification != null)
            Notification(
                modifier = notificationModifier,
                visibleNotification.toNotificationData(
                    isPlayBuild = isPlayBuild,
                    openAppListing = openAppListing,
                    onClickShowAccount = onClickShowAccount,
                    onClickShowChangelog = onClickShowChangelog,
                    onClickShowAndroid16UpgradeInfo = onClickShowAndroid16UpgradeInfo,
                    onClickDismissChangelog = onClickDismissChangelog,
                    onClickDismissNewDevice = onClickDismissNewDevice,
                    onClickShowWireguardPortSettings = onClickShowWireguardPortSettings,
                    onClickDismissAndroid16UpgradeWarning = onClickDismissAndroid16UpgradeWarning,
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
                .testTag(NOTIFICATION_BANNER_TEST_TAG)
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
            style = MaterialTheme.typography.labelLarge,
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
                                    .testTag(NOTIFICATION_BANNER_TEXT_ACTION_TEST_TAG)
                            } else {
                                it
                            }
                        },
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                style = MaterialTheme.typography.bodyMedium,
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
                            StatusLevel.None -> Color.Transparent
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
        modifier = modifier.testTag(NOTIFICATION_BANNER_ACTION_TEST_TAG),
        onClick = onClick,
    ) {
        Icon(
            modifier = Modifier.padding(Dimens.smallPadding),
            imageVector = imageVector,
            contentDescription = contentDescription,
            tint = MaterialTheme.colorScheme.onSurface,
        )
    }
}
