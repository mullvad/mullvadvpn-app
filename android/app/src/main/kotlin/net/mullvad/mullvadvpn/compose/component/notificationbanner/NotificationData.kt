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
import java.net.InetAddress
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.extensions.getExpiryQuantityString
import net.mullvad.mullvadvpn.compose.extensions.toAnnotatedString
import net.mullvad.mullvadvpn.lib.model.AuthFailedError
import net.mullvad.mullvadvpn.lib.model.ErrorState
import net.mullvad.mullvadvpn.lib.model.ErrorStateCause
import net.mullvad.mullvadvpn.lib.model.ParameterGenerationError
import net.mullvad.mullvadvpn.lib.model.PrepareError
import net.mullvad.mullvadvpn.repository.InAppNotification
import net.mullvad.mullvadvpn.ui.notification.StatusLevel

data class NotificationData(
    val title: AnnotatedString,
    val message: AnnotatedString? = null,
    val statusLevel: StatusLevel,
    val action: NotificationAction? = null,
) {
    constructor(
        title: String,
        message: String? = null,
        statusLevel: StatusLevel,
        action: NotificationAction? = null,
    ) : this(AnnotatedString(title), message?.let { AnnotatedString(it) }, statusLevel, action)
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
                title =
                    AnnotatedString(stringResource(id = R.string.new_device_notification_title)),
                message =
                    stringResource(id = R.string.new_device_notification_message, deviceName)
                        .formatWithHtml(),
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
    NotificationData(
        title = error.title().formatWithHtml(),
        message = error.message().formatWithHtml(),
        statusLevel = StatusLevel.Error,
        action = null,
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
        cause is ErrorStateCause.VpnPermissionDenied -> cause.prepareError.errorTitle()
        isBlocking -> stringResource(R.string.blocking_internet)
        else -> stringResource(R.string.critical_error)
    }
}

@Composable
private fun ErrorState.message(): String {
    val cause = this.cause
    return when {
        cause is ErrorStateCause.InvalidDnsServers -> {
            stringResource(
                cause.errorMessageId(),
                cause.addresses.joinToString { address -> address.addressString() },
            )
        }
        cause is ErrorStateCause.VpnPermissionDenied ->
            cause.prepareError.errorNotificationMessage()

        isBlocking -> stringResource(cause.errorMessageId())
        else -> stringResource(R.string.failed_to_block_internet)
    }
}

@Composable
private fun PrepareError.errorTitle(): String =
    when (this) {
        is PrepareError.NotPrepared ->
            stringResource(R.string.vpn_permission_error_notification_title)
        is PrepareError.OtherAlwaysOnApp ->
            stringResource(R.string.always_on_vpn_error_notification_title, appName)
        is PrepareError.LegacyLockdown ->
            stringResource(R.string.legacy_always_on_vpn_error_notification_title)
    }

@Composable
private fun PrepareError.errorNotificationMessage(): String =
    when (this) {
        is PrepareError.NotPrepared ->
            stringResource(R.string.vpn_permission_error_notification_message)
        is PrepareError.OtherAlwaysOnApp ->
            stringResource(R.string.always_on_vpn_error_notification_content, appName)
        is PrepareError.LegacyLockdown ->
            stringResource(R.string.legacy_always_on_vpn_error_notification_content)
    }

private fun ErrorStateCause.errorMessageId(): Int =
    when (this) {
        is ErrorStateCause.InvalidDnsServers -> R.string.invalid_dns_servers
        is ErrorStateCause.AuthFailed -> error.errorMessageId()
        is ErrorStateCause.Ipv6Unavailable -> R.string.ipv6_unavailable
        is ErrorStateCause.FirewallPolicyError -> R.string.set_firewall_policy_error
        is ErrorStateCause.DnsError -> R.string.set_dns_error
        is ErrorStateCause.StartTunnelError -> R.string.start_tunnel_error
        is ErrorStateCause.IsOffline -> R.string.is_offline
        is ErrorStateCause.TunnelParameterError -> {
            when (error) {
                ParameterGenerationError.NoMatchingRelay,
                ParameterGenerationError.NoMatchingBridgeRelay -> {
                    R.string.no_matching_relay
                }
                ParameterGenerationError.NoWireguardKey -> R.string.no_wireguard_key
                ParameterGenerationError.CustomTunnelHostResultionError -> {
                    R.string.custom_tunnel_host_resolution_error
                }
            }
        }
        is ErrorStateCause.VpnPermissionDenied -> R.string.vpn_permission_denied_error
    }

private fun AuthFailedError.errorMessageId(): Int =
    when (this) {
        AuthFailedError.ExpiredAccount -> R.string.account_credit_has_expired
        AuthFailedError.InvalidAccount,
        AuthFailedError.TooManyConnections,
        AuthFailedError.Unknown -> R.string.auth_failed
    }

private fun InetAddress.addressString(): String {
    val hostNameAndAddress = this.toString().split('/', limit = 2)
    val address = hostNameAndAddress[1]

    return address
}
