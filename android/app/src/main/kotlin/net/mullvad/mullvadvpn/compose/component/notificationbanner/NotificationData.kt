package net.mullvad.mullvadvpn.compose.component.notificationbanner

import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.OpenInNew
import androidx.compose.material.icons.filled.Clear
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.core.text.HtmlCompat
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.extensions.getExpiryQuantityString
import net.mullvad.mullvadvpn.compose.extensions.toAnnotatedString
import net.mullvad.mullvadvpn.lib.common.util.notificationResources
import net.mullvad.mullvadvpn.lib.model.ErrorState
import net.mullvad.mullvadvpn.repository.InAppNotification
import net.mullvad.mullvadvpn.ui.notification.StatusLevel

data class NotificationData(
    val title: String,
    val message: AnnotatedString? = null,
    val statusLevel: StatusLevel,
    val action: NotificationAction? = null,
) {
    constructor(
        title: String,
        message: String?,
        statusLevel: StatusLevel,
        action: NotificationAction?,
    ) : this(title, message?.let { AnnotatedString(it) }, statusLevel, action)
}

data class NotificationAction(
    val icon: ImageVector,
    val onClick: (() -> Unit),
    val contentDescription: String,
)

@Composable
fun InAppNotification.toNotificationData(
    isPlayBuild: Boolean,
    openAppListing: () -> Unit,
    onClickShowAccount: () -> Unit,
    onDismissNewDevice: () -> Unit,
) =
    when (this) {
        is InAppNotification.NewDevice ->
            NotificationData(
                title = stringResource(id = R.string.new_device_notification_title),
                message =
                    HtmlCompat.fromHtml(
                            stringResource(
                                id = R.string.new_device_notification_message,
                                deviceName,
                            ),
                            HtmlCompat.FROM_HTML_MODE_COMPACT,
                        )
                        .toAnnotatedString(
                            boldSpanStyle =
                                SpanStyle(
                                    color = MaterialTheme.colorScheme.onSurface,
                                    fontWeight = FontWeight.ExtraBold,
                                )
                        ),
                statusLevel = StatusLevel.Info,
                action =
                    NotificationAction(
                        Icons.Default.Clear,
                        onDismissNewDevice,
                        stringResource(id = R.string.dismiss),
                    ),
            )
        is InAppNotification.AccountExpiry ->
            NotificationData(
                title = stringResource(id = R.string.account_credit_expires_soon),
                message = LocalContext.current.resources.getExpiryQuantityString(expiry),
                statusLevel = StatusLevel.Error,
                action =
                    if (isPlayBuild) null
                    else
                        NotificationAction(
                            Icons.AutoMirrored.Default.OpenInNew,
                            onClickShowAccount,
                            stringResource(id = R.string.open_url),
                        ),
            )
        InAppNotification.TunnelStateBlocked ->
            NotificationData(
                title = stringResource(id = R.string.blocking_internet),
                statusLevel = StatusLevel.Error,
            )
        is InAppNotification.TunnelStateError -> errorMessageBannerData(error)
        is InAppNotification.UnsupportedVersion ->
            NotificationData(
                title = stringResource(id = R.string.unsupported_version),
                message = stringResource(id = R.string.unsupported_version_description),
                statusLevel = StatusLevel.Error,
                action =
                    NotificationAction(
                        Icons.AutoMirrored.Default.OpenInNew,
                        openAppListing,
                        stringResource(id = R.string.open_url),
                    ),
            )
    }

@Composable
private fun errorMessageBannerData(error: ErrorState) =
    with(error.notificationResources()) {
        NotificationData(
            title = stringResource(id = titleResourceId),
            message =
                HtmlCompat.fromHtml(
                        optionalMessageArgument?.let { stringResource(id = messageResourceId, it) }
                            ?: stringResource(id = messageResourceId),
                        HtmlCompat.FROM_HTML_MODE_COMPACT,
                    )
                    .toAnnotatedString(
                        boldSpanStyle =
                            SpanStyle(
                                color = MaterialTheme.colorScheme.onSurface,
                                fontWeight = FontWeight.ExtraBold,
                            )
                    ),
            statusLevel = StatusLevel.Error,
            action = null,
        )
    }
