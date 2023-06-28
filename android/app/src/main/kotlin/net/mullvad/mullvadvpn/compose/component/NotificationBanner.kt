package net.mullvad.mullvadvpn.compose.component

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.slideInVertically
import androidx.compose.animation.slideOutVertically
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.tooling.preview.Preview
import androidx.constraintlayout.compose.ConstraintLayout
import androidx.constraintlayout.compose.Dimension
import net.mullvad.mullvadvpn.BuildConfig
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.extensions.getExpiryQuantityString
import net.mullvad.mullvadvpn.compose.state.ConnectNotificationState
import net.mullvad.mullvadvpn.compose.util.rememberPrevious
import net.mullvad.mullvadvpn.lib.common.constant.BuildTypes
import net.mullvad.mullvadvpn.lib.common.util.getErrorNotificationResources
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.ui.VersionInfo
import net.mullvad.mullvadvpn.ui.notification.StatusLevel
import net.mullvad.talpid.tunnel.ErrorState
import org.joda.time.DateTime

@Preview
@Composable
fun PreviewNotificationBanner() {
    AppTheme {
        SpacedColumn(Modifier.background(color = MaterialTheme.colorScheme.background)) {
            VersionInfoNotification(
                modifier = Modifier,
                versionInfo =
                    VersionInfo(
                        currentVersion = null,
                        upgradeVersion = null,
                        isOutdated = true,
                        isSupported = false
                    ),
                onClickUpdateVersion = {}
            )
            AccountExpiryNotification(expiry = DateTime.now(), onClickShowAccount = {})
            GenericBlockingMessage()
        }
    }
}

@Composable
fun Notification(
    modifier: Modifier = Modifier,
    connectNotificationState: ConnectNotificationState,
    onClickUpdateVersion: () -> Unit,
    onClickShowAccount: () -> Unit
) {
    val isVisible = connectNotificationState != ConnectNotificationState.HideNotification
    // Fix for animating to hide
    val lastState: ConnectNotificationState =
        rememberPrevious(connectNotificationState) ?: ConnectNotificationState.HideNotification
    AnimatedVisibility(
        visible = isVisible,
        enter = slideInVertically(),
        exit = slideOutVertically()
    ) {
        ShowNotification(
            connectNotificationState = if (isVisible) connectNotificationState else lastState,
            modifier = modifier,
            onClickUpdateVersion = onClickUpdateVersion,
            onClickShowAccount = onClickShowAccount
        )
    }
}

@Composable
private fun ShowNotification(
    modifier: Modifier = Modifier,
    connectNotificationState: ConnectNotificationState,
    onClickUpdateVersion: () -> Unit,
    onClickShowAccount: () -> Unit
) {
    when (connectNotificationState) {
        ConnectNotificationState.ShowTunnelStateNotificationBlocked -> {
            GenericBlockingMessage(modifier = modifier)
        }
        is ConnectNotificationState.ShowTunnelStateNotificationError -> {
            ErrorMessage(modifier = modifier, error = connectNotificationState.error)
        }
        is ConnectNotificationState.ShowVersionInfoNotification -> {
            VersionInfoNotification(
                modifier = modifier,
                versionInfo = connectNotificationState.versionInfo,
                onClickUpdateVersion =
                    if (BuildConfig.BUILD_TYPE != BuildTypes.RELEASE) {
                        onClickUpdateVersion
                    } else {
                        null
                    }
            )
        }
        is ConnectNotificationState.ShowAccountExpiryNotification -> {
            AccountExpiryNotification(
                modifier = modifier,
                expiry = connectNotificationState.expiry,
                onClickShowAccount =
                    if (BuildConfig.BUILD_TYPE != BuildTypes.RELEASE) {
                        onClickShowAccount
                    } else {
                        null
                    }
            )
        }
        else -> {}
    }
}

@Composable
private fun NotificationBanner(
    modifier: Modifier = Modifier,
    title: String,
    message: String?,
    statusLevel: StatusLevel,
    onClick: (() -> Unit)? = null
) {
    ConstraintLayout(
        modifier =
            modifier
                .fillMaxWidth()
                .background(color = MaterialTheme.colorScheme.background)
                .padding(
                    start = Dimens.notificationBannerStartPadding,
                    end = Dimens.notificationBannerEndPadding,
                    top = Dimens.smallPadding,
                    bottom = Dimens.smallPadding
                )
                .then(onClick?.let { Modifier.clickable(onClick = onClick) } ?: Modifier)
    ) {
        val (status, textTitle, textMessage, icon) = createRefs()
        Box(
            modifier =
                Modifier.background(
                        color =
                            if (statusLevel == StatusLevel.Warning) {
                                MaterialTheme.colorScheme.errorContainer
                            } else {
                                MaterialTheme.colorScheme.error
                            },
                        shape = CircleShape
                    )
                    .size(Dimens.notificationStatusIconSize)
                    .constrainAs(status) {
                        top.linkTo(textTitle.top)
                        start.linkTo(parent.start)
                        bottom.linkTo(textTitle.bottom)
                    }
        ) {}
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
            fontWeight = FontWeight.Bold
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
        onClick?.let {
            Image(
                painter = painterResource(id = R.drawable.icon_extlink),
                contentDescription = null,
                modifier =
                    Modifier.constrainAs(icon) {
                            top.linkTo(parent.top)
                            end.linkTo(parent.end)
                            bottom.linkTo(parent.bottom)
                        }
                        .padding(all = Dimens.notificationEndIconPadding)
            )
        }
    }
}

@Composable
private fun GenericBlockingMessage(
    modifier: Modifier = Modifier,
) {
    NotificationBanner(
        modifier = modifier,
        title = stringResource(id = R.string.blocking_internet),
        message = null,
        statusLevel = StatusLevel.Error
    )
}

@Composable
private fun ErrorMessage(modifier: Modifier = Modifier, error: ErrorState) {
    error.getErrorNotificationResources(LocalContext.current).apply {
        NotificationBanner(
            title = stringResource(id = titleResourceId),
            message = optionalMessageArgument?.let { stringResource(id = messageResourceId, it) }
                    ?: stringResource(id = messageResourceId),
            modifier = modifier,
            statusLevel = StatusLevel.Error
        )
    }
}

@Composable
private fun AccountExpiryNotification(
    modifier: Modifier = Modifier,
    expiry: DateTime,
    onClickShowAccount: (() -> Unit)?
) {
    NotificationBanner(
        modifier,
        stringResource(id = R.string.account_credit_expires_soon),
        LocalContext.current.resources.getExpiryQuantityString(expiry),
        statusLevel = StatusLevel.Error,
        onClick = onClickShowAccount
    )
}

@Composable
private fun VersionInfoNotification(
    modifier: Modifier,
    versionInfo: VersionInfo,
    onClickUpdateVersion: (() -> Unit)?
) {
    when {
        versionInfo.upgradeVersion != null && versionInfo.isSupported ->
            NotificationBanner(
                title = stringResource(id = R.string.update_available),
                message =
                    stringResource(
                        id = R.string.update_available_description,
                        versionInfo.upgradeVersion
                    ),
                statusLevel = StatusLevel.Warning,
                modifier = modifier,
                onClick = onClickUpdateVersion
            )
        else ->
            NotificationBanner(
                title = stringResource(id = R.string.unsupported_version),
                message = stringResource(id = R.string.unsupported_version_description),
                statusLevel = StatusLevel.Error,
                modifier = modifier,
                onClick = onClickUpdateVersion
            )
    }
}
