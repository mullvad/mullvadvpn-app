package net.mullvad.mullvadvpn.service.notifications

import android.app.Notification
import android.app.NotificationManager
import android.app.PendingIntent
import android.content.Context
import android.content.Intent
import androidx.core.app.NotificationCompat
import kotlin.properties.Delegates.observable
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.model.TunnelState
import net.mullvad.mullvadvpn.ui.MainActivity
import net.mullvad.mullvadvpn.util.SdkUtils
import net.mullvad.mullvadvpn.util.getErrorNotificationResources
import net.mullvad.talpid.tunnel.ActionAfterDisconnect

class TunnelStateNotification(val context: Context) {
    companion object {
        val NOTIFICATION_ID: Int = 1
    }

    private val channel = NotificationChannel(
        context,
        "vpn_tunnel_status",
        NotificationCompat.VISIBILITY_SECRET,
        R.string.foreground_notification_channel_name,
        R.string.foreground_notification_channel_description,
        NotificationManager.IMPORTANCE_MIN,
        false,
        false
    )

    private val notificationText: Int
        get() = when (val state = tunnelState) {
            is TunnelState.Disconnected -> R.string.unsecured
            is TunnelState.Connecting -> {
                if (reconnecting) {
                    R.string.reconnecting
                } else {
                    R.string.connecting
                }
            }
            is TunnelState.Connected -> R.string.secured
            is TunnelState.Disconnecting -> {
                when (state.actionAfterDisconnect) {
                    ActionAfterDisconnect.Reconnect -> R.string.reconnecting
                    else -> R.string.disconnecting
                }
            }
            is TunnelState.Error -> {
                state.errorState.getErrorNotificationResources(context).titleResourceId
            }
        }

    private val shouldDisplayOngoingNotification: Boolean
        get() = when (tunnelState) {
            is TunnelState.Connected -> true
            is TunnelState.Disconnected,
            is TunnelState.Connecting,
            is TunnelState.Disconnecting,
            is TunnelState.Error -> false
        }

    private var reconnecting = false
    private var showingReconnecting = false

    var showAction by observable(false) { _, _, _ -> update() }

    var tunnelState by observable<TunnelState>(TunnelState.Disconnected) { _, _, newState ->
        val isReconnecting = newState is TunnelState.Connecting && reconnecting
        val shouldBeginReconnecting = (newState as? TunnelState.Disconnecting)
            ?.actionAfterDisconnect == ActionAfterDisconnect.Reconnect
        reconnecting = isReconnecting || shouldBeginReconnecting
        update()
    }

    var visible by observable(true) { _, _, newValue ->
        if (newValue == true) {
            update()
        } else {
            channel.notificationManager.cancel(NOTIFICATION_ID)
        }
    }

    private fun update() {
        if (visible && (!reconnecting || !showingReconnecting)) {
            channel.notificationManager.notify(NOTIFICATION_ID, build())
        }
    }

    fun build(): Notification {
        val intent = Intent(context, MainActivity::class.java)
            .setFlags(Intent.FLAG_ACTIVITY_CLEAR_TOP or Intent.FLAG_ACTIVITY_SINGLE_TOP)
            .setAction(Intent.ACTION_MAIN)
        val pendingIntent = PendingIntent.getActivity(
            context,
            1,
            intent,
            SdkUtils.getSupportedPendingIntentFlags()
        )
        val actions = if (showAction) {
            listOf(buildAction())
        } else {
            emptyList()
        }

        return channel.buildNotification(
            pendingIntent,
            notificationText,
            actions,
            isOngoing = shouldDisplayOngoingNotification
        )
    }

    private fun buildAction(): NotificationCompat.Action {
        val action = TunnelStateNotificationAction.from(tunnelState)
        val label = context.getString(action.text)
        val intent = Intent(action.key).setPackage("net.mullvad.mullvadvpn")
        val pendingIntent = PendingIntent.getForegroundService(
            context,
            1,
            intent,
            SdkUtils.getSupportedPendingIntentFlags()
        )

        return NotificationCompat.Action(action.icon, label, pendingIntent)
    }
}
