package net.mullvad.mullvadvpn.service.notifications.tunnelstate

import android.app.PendingIntent
import android.content.Context
import android.content.Intent
import androidx.core.app.NotificationCompat
import net.mullvad.mullvadvpn.lib.common.constant.KEY_CONNECT_ACTION
import net.mullvad.mullvadvpn.lib.common.constant.KEY_DISCONNECT_ACTION
import net.mullvad.mullvadvpn.lib.common.constant.KEY_REQUEST_VPN_PROFILE
import net.mullvad.mullvadvpn.lib.common.constant.MAIN_ACTIVITY_CLASS
import net.mullvad.mullvadvpn.lib.common.util.SdkUtils
import net.mullvad.mullvadvpn.lib.model.Notification
import net.mullvad.mullvadvpn.lib.model.NotificationAction
import net.mullvad.mullvadvpn.lib.model.NotificationTunnelState
import net.mullvad.mullvadvpn.lib.model.PrepareError
import net.mullvad.mullvadvpn.service.R

internal fun Notification.Tunnel.toNotification(context: Context) =
    NotificationCompat.Builder(context, channelId.value)
        .setContentIntent(contentIntent(context))
        .setContentTitle(state.contentTitleResourceId(context))
        .setContentText(state.contentText())
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

private fun NotificationTunnelState.contentTitleResourceId(context: Context): String =
    when (this) {
        is NotificationTunnelState.Connected -> context.getString(R.string.connected)
        is NotificationTunnelState.Connecting -> context.getString(R.string.connecting)
        is NotificationTunnelState.Disconnected -> {
            when (prepareError) {
                is PrepareError.NotPrepared ->
                    context.getString(R.string.disconnected_vpn_permission_error)
                else -> context.getString(R.string.disconnected)
            }
        }
        NotificationTunnelState.Disconnecting -> context.getString(R.string.disconnecting)
        NotificationTunnelState.Blocking -> context.getString(R.string.blocking)
        NotificationTunnelState.Error.Blocked -> context.getString(R.string.blocking_internet)
        is NotificationTunnelState.Error.Critical -> context.getString(R.string.critical_error)
        NotificationTunnelState.Error.DeviceOffline ->
            context.getString(R.string.blocking_internet_device_offline)
        NotificationTunnelState.Error.VpnPermissionDenied ->
            context.getString(R.string.vpn_permission_error_notification_title)
        is NotificationTunnelState.Error.AlwaysOnVpn ->
            context.getString(R.string.always_on_vpn_error_notification_title, appName)
        NotificationTunnelState.Error.LegacyLockdown ->
            context.getString(R.string.legacy_always_on_vpn_error_notification_title)
    }

private fun NotificationTunnelState.contentText(): CharSequence? {
    val location =
        when (this) {
            is NotificationTunnelState.Connected -> location
            is NotificationTunnelState.Connecting -> location
            else -> null
        }
    return if (location != null) {
        "${location.country}, ${location.city}, ${location.hostname}"
    } else {
        null
    }
}

internal fun NotificationAction.Tunnel.toCompatAction(context: Context): NotificationCompat.Action {

    val pendingIntent =
        if (this is NotificationAction.Tunnel.RequestVpnProfile) {
            val intent =
                Intent().apply {
                    setClassName(context.packageName, MAIN_ACTIVITY_CLASS)
                    addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
                    addFlags(Intent.FLAG_ACTIVITY_CLEAR_TOP)
                    setAction(KEY_REQUEST_VPN_PROFILE)
                }

            PendingIntent.getActivity(context, 1, intent, SdkUtils.getSupportedPendingIntentFlags())
        } else {
            val intent = Intent(toKey()).setPackage(context.packageName)
            PendingIntent.getService(context, 1, intent, SdkUtils.getSupportedPendingIntentFlags())
        }

    return NotificationCompat.Action(
        toIconResource(),
        context.getString(titleResource()),
        pendingIntent,
    )
}

fun NotificationAction.Tunnel.titleResource() =
    when (this) {
        NotificationAction.Tunnel.Cancel -> R.string.cancel
        NotificationAction.Tunnel.Connect,
        is NotificationAction.Tunnel.RequestVpnProfile -> R.string.connect
        NotificationAction.Tunnel.Disconnect -> R.string.disconnect
        NotificationAction.Tunnel.Dismiss -> R.string.dismiss
    }

fun NotificationAction.Tunnel.toKey() =
    when (this) {
        NotificationAction.Tunnel.Connect -> KEY_CONNECT_ACTION
        is NotificationAction.Tunnel.RequestVpnProfile -> KEY_REQUEST_VPN_PROFILE
        NotificationAction.Tunnel.Cancel,
        NotificationAction.Tunnel.Disconnect,
        NotificationAction.Tunnel.Dismiss -> KEY_DISCONNECT_ACTION
    }

fun NotificationAction.Tunnel.toIconResource() =
    when (this) {
        NotificationAction.Tunnel.Connect -> R.drawable.icon_notification_connect
        else -> R.drawable.icon_notification_disconnect
    }
