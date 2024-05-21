package net.mullvad.mullvadvpn.service.notifications.tunnelstate

import android.app.PendingIntent
import android.content.Context
import android.content.Intent
import androidx.core.app.NotificationCompat
import net.mullvad.mullvadvpn.lib.common.constant.KEY_CONNECT_ACTION
import net.mullvad.mullvadvpn.lib.common.constant.KEY_DISCONNECT_ACTION
import net.mullvad.mullvadvpn.lib.common.constant.KEY_REQUEST_VPN_PERMISSION
import net.mullvad.mullvadvpn.lib.common.constant.MAIN_ACTIVITY_CLASS
import net.mullvad.mullvadvpn.lib.common.util.SdkUtils
import net.mullvad.mullvadvpn.lib.common.util.errorMessageId
import net.mullvad.mullvadvpn.model.Notification
import net.mullvad.mullvadvpn.model.NotificationAction
import net.mullvad.mullvadvpn.model.NotificationTunnelState
import net.mullvad.mullvadvpn.service.R

internal fun Notification.Tunnel.toNotification(context: Context) =
    NotificationCompat.Builder(context, channelId.value)
        .setContentIntent(contentIntent(context))
        .setContentTitle(context.getString(state.contentTitleResourceId()))
        .setSmallIcon(R.drawable.small_logo_white)
        .apply { actions.forEach { addAction(it.toCompatAction(context)) } }
        .setOngoing(ongoing)
        .setVisibility(NotificationCompat.VISIBILITY_SECRET)
        .build()

private fun Notification.Tunnel.contentIntent(context: Context): PendingIntent {
    val intent =
        Intent().apply {
            setClassName(context.packageName, MAIN_ACTIVITY_CLASS)
            flags = Intent.FLAG_ACTIVITY_CLEAR_TOP or Intent.FLAG_ACTIVITY_SINGLE_TOP
            action = Intent.ACTION_MAIN
        }

    return PendingIntent.getActivity(context, 1, intent, SdkUtils.getSupportedPendingIntentFlags())
}

private fun NotificationTunnelState.contentTitleResourceId(): Int =
    when (this) {
        NotificationTunnelState.Connected -> R.string.secured
        NotificationTunnelState.Connecting -> R.string.connecting
        is NotificationTunnelState.Disconnected -> {
            if (this.hasVpnPermission) {
                R.string.unsecured
            } else {
                R.string.unsecured_vpn_permission_error
            }
        }
        NotificationTunnelState.Disconnecting -> R.string.disconnecting
        NotificationTunnelState.Error.Blocking -> TODO()
        is NotificationTunnelState.Error.Critical -> this.cause.errorMessageId()
        NotificationTunnelState.Error.DeviceOffline -> R.string.blocking_internet_device_offline
        is NotificationTunnelState.Error.InvalidDnsServers -> TODO()
        NotificationTunnelState.Error.VpnPermissionDenied ->
            R.string.vpn_permission_error_notification_title
        NotificationTunnelState.Reconnecting -> R.string.reconnecting
    }

internal fun NotificationAction.Tunnel.toCompatAction(context: Context): NotificationCompat.Action {

    val pendingIntent =
        if (this is NotificationAction.Tunnel.RequestPermission) {
            val intent =
                Intent().apply {
                    setClassName(context.packageName, MAIN_ACTIVITY_CLASS)
                    addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
                    addFlags(Intent.FLAG_ACTIVITY_CLEAR_TOP)
                    setAction(KEY_REQUEST_VPN_PERMISSION)
                }

            PendingIntent.getActivity(context, 1, intent, SdkUtils.getSupportedPendingIntentFlags())
        } else {
            val intent = Intent(toKey()).setPackage(context.packageName)

            PendingIntent.getForegroundService(
                context,
                1,
                intent,
                SdkUtils.getSupportedPendingIntentFlags()
            )
        }

    return NotificationCompat.Action(
        toIconResource(),
        context.getString(titleResource()),
        pendingIntent
    )
}

fun NotificationAction.Tunnel.titleResource() =
    when (this) {
        NotificationAction.Tunnel.Cancel -> R.string.cancel
        NotificationAction.Tunnel.Connect,
        NotificationAction.Tunnel.RequestPermission -> R.string.connect
        NotificationAction.Tunnel.Disconnect -> R.string.disconnect
        NotificationAction.Tunnel.Dismiss -> R.string.dismiss
    }

fun NotificationAction.Tunnel.toKey() =
    when (this) {
        NotificationAction.Tunnel.Connect,
        NotificationAction.Tunnel.RequestPermission -> KEY_CONNECT_ACTION
        NotificationAction.Tunnel.Cancel,
        NotificationAction.Tunnel.Disconnect,
        NotificationAction.Tunnel.Dismiss -> KEY_DISCONNECT_ACTION
    }

fun NotificationAction.Tunnel.toIconResource() =
    when (this) {
        NotificationAction.Tunnel.Connect -> R.drawable.icon_notification_connect
        else -> R.drawable.icon_notification_disconnect
    }
