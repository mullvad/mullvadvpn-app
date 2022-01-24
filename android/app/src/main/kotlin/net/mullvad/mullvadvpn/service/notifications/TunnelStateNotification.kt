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
                if (state.errorState.isBlocking) {
                    R.string.blocking_all_connections
                } else {
                    R.string.critical_error
                }
            }
        }

    private var reconnecting = false
    private var showingReconnecting = false

    var showAction by observable(false) { _, _, _ -> update() }

    var tunnelState by observable<TunnelState>(TunnelState.Disconnected) { _, _, newState ->
        reconnecting =
            (
            newState is TunnelState.Disconnecting &&
                newState.actionAfterDisconnect == ActionAfterDisconnect.Reconnect
            ) ||
            (newState is TunnelState.Connecting && reconnecting)

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

        val pendingIntent =
            PendingIntent.getActivity(context, 1, intent, PendingIntent.FLAG_UPDATE_CURRENT)

        val actions = if (showAction) {
            listOf(buildAction())
        } else {
            emptyList()
        }

        return channel.buildNotification(pendingIntent, notificationText, actions)
    }

    private fun buildAction(): NotificationCompat.Action {
        val action = TunnelStateNotificationAction.from(tunnelState)
        val label = context.getString(action.text)

        val intent = Intent(action.key).setPackage("net.mullvad.mullvadvpn")
        val flags = PendingIntent.FLAG_UPDATE_CURRENT

        val pendingIntent = PendingIntent.getForegroundService(context, 1, intent, flags)

        return NotificationCompat.Action(action.icon, label, pendingIntent)
    }
}
