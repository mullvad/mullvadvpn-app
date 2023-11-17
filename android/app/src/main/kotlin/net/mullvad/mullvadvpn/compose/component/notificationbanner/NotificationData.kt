package net.mullvad.mullvadvpn.compose.component.notificationbanner

import androidx.annotation.DrawableRes
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.core.text.HtmlCompat
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.extensions.getExpiryQuantityString
import net.mullvad.mullvadvpn.compose.extensions.toAnnotatedString
import net.mullvad.mullvadvpn.constant.IS_PLAY_BUILD
import net.mullvad.mullvadvpn.lib.common.util.getErrorNotificationResources
import net.mullvad.mullvadvpn.repository.InAppNotification
import net.mullvad.mullvadvpn.ui.notification.StatusLevel
import net.mullvad.talpid.tunnel.ErrorState

data class NotificationData(
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

@Composable
fun InAppNotification.toNotificationData(
    onClickUpdateVersion: () -> Unit,
    onClickShowAccount: () -> Unit,
    onDismissNewDevice: () -> Unit
) =
    when (this) {
        is InAppNotification.NewDevice ->
            NotificationData(
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
                action = NotificationAction(R.drawable.icon_close, onDismissNewDevice)
            )
        is InAppNotification.AccountExpiry ->
            NotificationData(
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
        InAppNotification.TunnelStateBlocked ->
            NotificationData(
                title = stringResource(id = R.string.blocking_internet),
                statusLevel = StatusLevel.Error
            )
        is InAppNotification.TunnelStateError -> errorMessageBannerData(error)
        is InAppNotification.UnsupportedVersion ->
            NotificationData(
                title = stringResource(id = R.string.unsupported_version),
                message = stringResource(id = R.string.unsupported_version_description),
                statusLevel = StatusLevel.Error,
                action =
                    if (IS_PLAY_BUILD) null
                    else NotificationAction(R.drawable.icon_extlink, onClickUpdateVersion)
            )
        is InAppNotification.UpdateAvailable ->
            NotificationData(
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
        NotificationData(
            title = stringResource(id = titleResourceId),
            message =
                HtmlCompat.fromHtml(
                        optionalMessageArgument?.let { stringResource(id = messageResourceId, it) }
                            ?: stringResource(id = messageResourceId),
                        HtmlCompat.FROM_HTML_MODE_COMPACT
                    )
                    .toAnnotatedString(
                        boldSpanStyle =
                            SpanStyle(
                                color = MaterialTheme.colorScheme.onBackground,
                                fontWeight = FontWeight.ExtraBold
                            )
                    ),
            statusLevel = StatusLevel.Error,
            action = null
        )
    }
