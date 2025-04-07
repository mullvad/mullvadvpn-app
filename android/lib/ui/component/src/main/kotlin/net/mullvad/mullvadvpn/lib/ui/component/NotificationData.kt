package net.mullvad.mullvadvpn.lib.ui.component

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
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextDecoration
import androidx.compose.ui.text.withStyle
import androidx.core.text.HtmlCompat
import java.net.InetAddress
import net.mullvad.mullvadvpn.lib.model.AuthFailedError
import net.mullvad.mullvadvpn.lib.model.ErrorState
import net.mullvad.mullvadvpn.lib.model.ErrorStateCause
import net.mullvad.mullvadvpn.lib.model.InAppNotification
import net.mullvad.mullvadvpn.lib.model.ParameterGenerationError
import net.mullvad.mullvadvpn.lib.model.StatusLevel

data class NotificationData(
    val title: AnnotatedString,
    val message: NotificationMessage? = null,
    val statusLevel: StatusLevel,
    val action: NotificationAction? = null,
) {
    constructor(
        title: String,
        message: String? = null,
        statusLevel: StatusLevel,
        action: NotificationAction? = null,
    ) : this(
        AnnotatedString(title),
        message?.let { NotificationMessage.Text(AnnotatedString(it)) },
        statusLevel,
        action,
    )

    constructor(
        title: String,
        message: NotificationMessage,
        statusLevel: StatusLevel,
        action: NotificationAction? = null,
    ) : this(AnnotatedString(title), message, statusLevel, action)
}

sealed interface NotificationMessage {
    val text: AnnotatedString

    data class Text(override val text: AnnotatedString) : NotificationMessage

    data class ClickableText(
        override val text: AnnotatedString,
        val onClick: () -> Unit,
        val contentDescription: String,
    ) : NotificationMessage
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
    onClickShowChangelog: () -> Unit,
    onClickDismissChangelog: () -> Unit,
    onClickDismissNewDevice: () -> Unit,
) =
    when (this) {
        is InAppNotification.NewDevice ->
            NotificationData(
                title =
                    AnnotatedString(stringResource(id = R.string.new_device_notification_title)),
                message =
                    NotificationMessage.Text(
                        stringResource(id = R.string.new_device_notification_message, deviceName)
                            .formatWithHtml()
                    ),
                statusLevel = StatusLevel.Info,
                action =
                    NotificationAction(
                        Icons.Default.Clear,
                        onClickDismissNewDevice,
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
        is InAppNotification.NewVersionChangelog ->
            NotificationData(
                title = stringResource(id = R.string.new_changelog_notification_title),
                message =
                    NotificationMessage.ClickableText(
                        text =
                            buildAnnotatedString {
                                withStyle(SpanStyle(textDecoration = TextDecoration.Underline)) {
                                    append(
                                        stringResource(
                                            id = R.string.new_changelog_notification_message
                                        )
                                    )
                                }
                            },
                        onClick = onClickShowChangelog,
                        contentDescription =
                            stringResource(id = R.string.new_changelog_notification_message),
                    ),
                statusLevel = StatusLevel.Info,
                action =
                    NotificationAction(
                        Icons.Default.Clear,
                        onClickDismissChangelog,
                        stringResource(id = R.string.dismiss),
                    ),
            )
    }

@Composable
private fun errorMessageBannerData(error: ErrorState) =
    NotificationData(
        title = error.title().formatWithHtml(),
        message = NotificationMessage.Text(error.message().formatWithHtml()),
        statusLevel = StatusLevel.Error,
    )

@Composable
private fun String.formatWithHtml(): AnnotatedString =
    HtmlCompat.fromHtml(this, HtmlCompat.FROM_HTML_MODE_COMPACT)
        .toAnnotatedString(
            boldSpanStyle =
                SpanStyle(
                    color = MaterialTheme.colorScheme.onSurface,
                    fontWeight = FontWeight.ExtraBold,
                )
        )

@Composable
private fun ErrorState.title(): String {
    val cause = this.cause
    return when {
        cause is ErrorStateCause.InvalidDnsServers -> stringResource(R.string.blocking_internet)
        cause is ErrorStateCause.NotPrepared ->
            stringResource(R.string.vpn_permission_error_notification_title)
        cause is ErrorStateCause.OtherAlwaysOnApp ->
            stringResource(R.string.always_on_vpn_error_notification_title, cause.appName)
        cause is ErrorStateCause.OtherLegacyAlwaysOnApp ->
            stringResource(R.string.legacy_always_on_vpn_error_notification_title)
        isBlocking -> stringResource(R.string.blocking_internet)
        else -> stringResource(R.string.critical_error)
    }
}

@Composable
private fun ErrorState.message(): String {
    val cause = this.cause
    return when {
        isBlocking -> cause.errorMessageId()
        else -> stringResource(R.string.failed_to_block_internet)
    }
}

@Composable
private fun ErrorStateCause.errorMessageId(): String =
    when (this) {
        is ErrorStateCause.AuthFailed -> stringResource(error.errorMessageId())
        is ErrorStateCause.Ipv6Unavailable -> stringResource(R.string.ipv6_unavailable)
        is ErrorStateCause.FirewallPolicyError -> stringResource(R.string.set_firewall_policy_error)
        is ErrorStateCause.DnsError -> stringResource(R.string.set_dns_error)
        is ErrorStateCause.StartTunnelError -> stringResource(R.string.start_tunnel_error)
        is ErrorStateCause.IsOffline -> stringResource(R.string.is_offline)
        is ErrorStateCause.TunnelParameterError -> stringResource(error.errorMessageId())
        is ErrorStateCause.NotPrepared ->
            stringResource(R.string.vpn_permission_error_notification_message)
        is ErrorStateCause.OtherAlwaysOnApp ->
            stringResource(R.string.always_on_vpn_error_notification_content, appName)
        is ErrorStateCause.OtherLegacyAlwaysOnApp ->
            stringResource(R.string.legacy_always_on_vpn_error_notification_content)
        is ErrorStateCause.InvalidDnsServers ->
            stringResource(
                R.string.invalid_dns_servers,
                addresses.joinToString { address -> address.addressString() },
            )
    }

private fun AuthFailedError.errorMessageId(): Int =
    when (this) {
        AuthFailedError.ExpiredAccount -> R.string.account_credit_has_expired
        AuthFailedError.InvalidAccount,
        AuthFailedError.TooManyConnections,
        AuthFailedError.Unknown -> R.string.auth_failed
    }

private fun ParameterGenerationError.errorMessageId(): Int =
    when (this) {
        ParameterGenerationError.NoMatchingRelay,
        ParameterGenerationError.NoMatchingBridgeRelay -> {
            R.string.no_matching_relay
        }
        ParameterGenerationError.NoWireguardKey -> R.string.no_wireguard_key
        ParameterGenerationError.CustomTunnelHostResolutionError ->
            R.string.custom_tunnel_host_resolution_error
        ParameterGenerationError.IpVersionUnavailable -> R.string.ip_version_unavailable
    }

private fun InetAddress.addressString(): String {
    val hostNameAndAddress = this.toString().split('/', limit = 2)
    val address = hostNameAndAddress[1]

    return address
}
