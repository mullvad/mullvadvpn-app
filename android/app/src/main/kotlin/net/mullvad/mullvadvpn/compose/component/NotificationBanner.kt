package net.mullvad.mullvadvpn.compose.component

import androidx.annotation.DrawableRes
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.animateContentSize
import androidx.compose.animation.slideInVertically
import androidx.compose.animation.slideOutVertically
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
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
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.constraintlayout.compose.ConstraintLayout
import androidx.constraintlayout.compose.Dimension
import androidx.core.text.HtmlCompat
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.extensions.getExpiryQuantityString
import net.mullvad.mullvadvpn.compose.extensions.toAnnotatedString
import net.mullvad.mullvadvpn.compose.test.NOTIFICATION_BANNER
import net.mullvad.mullvadvpn.constant.IS_PLAY_BUILD
import net.mullvad.mullvadvpn.lib.common.util.getErrorNotificationResources
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.repository.InAppNotification
import net.mullvad.mullvadvpn.ui.VersionInfo
import net.mullvad.mullvadvpn.ui.notification.StatusLevel
import net.mullvad.talpid.tunnel.ErrorState
import org.joda.time.DateTime

@Preview
@Composable
private fun PreviewNotificationBanner() {
    AppTheme {
        SpacedColumn(
            Modifier.background(color = MaterialTheme.colorScheme.background),
            spacing = 8.dp
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
                        InAppNotification.AccountExpiryNotification(expiry = DateTime.now()),
                        InAppNotification.ShowTunnelStateBlockedNotification,
                        InAppNotification.NewDeviceNotification("Courageous Turtle") {},
                    )
                    .map { it.toNotificationData({}, {}, {}) }

            bannerDataList.forEach { NotificationBanner(it) }
        }
    }
}

@Composable
fun Notification(
    notification: InAppNotification?,
    onClickUpdateVersion: () -> Unit,
    onClickShowAccount: () -> Unit,
    onClickDismissNewDevice: () -> Unit
) {
    // Fix for animating to hide
    AnimatedVisibility(
        visible = notification != null,
        enter = slideInVertically(),
        exit = slideOutVertically(),
        modifier = Modifier.animateContentSize()
    ) {
        if (notification == null) return@AnimatedVisibility
        ShowNotification(
            notification = notification,
            onClickUpdateVersion = onClickUpdateVersion,
            onClickShowAccount = onClickShowAccount,
            onDismiss = onClickDismissNewDevice
        )
    }
}

@Composable
private fun ShowNotification(
    notification: InAppNotification,
    onClickUpdateVersion: () -> Unit,
    onClickShowAccount: () -> Unit,
    onDismiss: () -> Unit
) {
    val notificationData =
        notification.toNotificationData(onClickUpdateVersion, onClickShowAccount, onDismiss)
    NotificationBanner(notificationData)
}

@Composable
private fun NotificationBanner(notificationBannerData: NotificationBannerData) {
    val (title, message, statusLevel, action) = notificationBannerData
    ConstraintLayout(
        modifier =
            Modifier.fillMaxWidth()
                .background(color = MaterialTheme.colorScheme.background)
                .then(action?.let { Modifier.clickable(onClick = action.onClick) } ?: Modifier)
                .padding(
                    start = Dimens.notificationBannerStartPadding,
                    end = Dimens.notificationBannerEndPadding,
                    top = Dimens.smallPadding,
                    bottom = Dimens.smallPadding
                )
                .animateContentSize()
                .testTag(NOTIFICATION_BANNER)
    ) {
        val (status, textTitle, textMessage, icon) = createRefs()
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
                        end.linkTo(icon.start)
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
                            end.linkTo(icon.start)
                            width = Dimension.fillToConstraints
                        }
                        .padding(start = Dimens.smallPadding),
                style = MaterialTheme.typography.labelMedium
            )
        }
        action?.let {
            IconButton(
                modifier =
                    Modifier.constrainAs(icon) {
                            top.linkTo(parent.top)
                            end.linkTo(parent.end)
                            bottom.linkTo(parent.bottom)
                        }
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

@Composable
fun InAppNotification.toNotificationData(
    onClickUpdateVersion: () -> Unit,
    onClickShowAccount: () -> Unit,
    onDismiss: () -> Unit
) =
    when (this) {
        is InAppNotification.NewDeviceNotification ->
            NotificationBannerData(
                title = stringResource(id = R.string.new_device_notification_title),
                message =
                    HtmlCompat.fromHtml(
                            stringResource(
                                id = R.string.new_device_notification_message,
                                deviceName
                            ),
                            HtmlCompat.FROM_HTML_MODE_COMPACT
                        )
                        .toAnnotatedString(
                            boldSpanStyle =
                                SpanStyle(
                                    color = MaterialTheme.colorScheme.onBackground,
                                    fontWeight = FontWeight.ExtraBold
                                ),
                        ),
                statusLevel = StatusLevel.Info,
                action = NotificationAction(R.drawable.icon_close, dismiss)
            )
        is InAppNotification.AccountExpiryNotification ->
            NotificationBannerData(
                title = stringResource(id = R.string.account_credit_expires_soon),
                message = LocalContext.current.resources.getExpiryQuantityString(expiry),
                statusLevel = StatusLevel.Error,
                action =
                    if (IS_PLAY_BUILD) null
                    else
                        NotificationAction(
                            R.drawable.icon_extlink,
                            onClickShowAccount,
                        ),
            )
        InAppNotification.ShowTunnelStateBlockedNotification ->
            NotificationBannerData(
                title = stringResource(id = R.string.blocking_internet),
                statusLevel = StatusLevel.Error
            )
        is InAppNotification.ShowTunnelStateErrorNotification -> errorMessageBannerData(error)
        is InAppNotification.UnsupportedVersion ->
            NotificationBannerData(
                title = stringResource(id = R.string.unsupported_version),
                message = stringResource(id = R.string.unsupported_version_description),
                statusLevel = StatusLevel.Error,
                action =
                    if (IS_PLAY_BUILD) null
                    else NotificationAction(R.drawable.icon_extlink, onClickUpdateVersion)
            )
        is InAppNotification.UpdateAvailable ->
            NotificationBannerData(
                title = stringResource(id = R.string.update_available),
                message =
                    stringResource(
                        id = R.string.update_available_description,
                        versionInfo.upgradeVersion ?: "" // TODO Verify
                    ),
                statusLevel = StatusLevel.Warning,
                action =
                    if (IS_PLAY_BUILD) null
                    else NotificationAction(R.drawable.icon_extlink, onClickUpdateVersion)
            )
    }

@Composable
private fun errorMessageBannerData(error: ErrorState) =
    error.getErrorNotificationResources(LocalContext.current).run {
        NotificationBannerData(
            title = stringResource(id = titleResourceId),
            message =
                optionalMessageArgument?.let { stringResource(id = messageResourceId, it) }
                    ?: stringResource(id = messageResourceId),
            statusLevel = StatusLevel.Error,
            action = null
        )
    }

data class NotificationBannerData(
    val title: String,
    val message: AnnotatedString? = null,
    val statusLevel: StatusLevel,
    val action: NotificationAction? = null
) {
    constructor(
        title: String,
        message: String?,
        statusLevel: StatusLevel,
        action: NotificationAction?
    ) : this(title, message?.let { AnnotatedString(it) }, statusLevel, action)
}

data class NotificationAction(
    @DrawableRes val icon: Int,
    val onClick: (() -> Unit),
)
