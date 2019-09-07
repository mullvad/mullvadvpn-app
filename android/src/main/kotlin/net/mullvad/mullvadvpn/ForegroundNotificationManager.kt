package net.mullvad.mullvadvpn

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.app.Service
import android.content.Context
import android.content.Intent
import android.os.Build
import android.support.v4.app.NotificationCompat

import net.mullvad.mullvadvpn.dataproxy.ConnectionProxy
import net.mullvad.mullvadvpn.model.ActionAfterDisconnect
import net.mullvad.mullvadvpn.model.TunnelState

val CHANNEL_ID = "vpn_tunnel_status"
val FOREGROUND_NOTIFICATION_ID: Int = 1

class ForegroundNotificationManager(val service: Service, val connectionProxy: ConnectionProxy) {
    private var listenerId: Int? = null
    private var reconnecting = false
    private var showingReconnecting = false

    private lateinit var notificationManager: NotificationManager

    private var tunnelState: TunnelState = TunnelState.Disconnected()
        set(value) {
            field = value

            reconnecting =
                (value is TunnelState.Disconnecting
                    && value.actionAfterDisconnect is ActionAfterDisconnect.Reconnect)
                || (value is TunnelState.Connecting && reconnecting)

            updateNotification()
        }

    private val notificationText: Int
        get() {
            val state = tunnelState

            return when (state) {
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
                        is ActionAfterDisconnect.Reconnect -> R.string.reconnecting
                        else -> R.string.disconnecting
                    }
                }
                is TunnelState.Blocked -> R.string.blocking_all_connections
            }
        }

    fun onCreate() {
        notificationManager =
            service.getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager

        listenerId = connectionProxy.onUiStateChange.subscribe { uiState ->
            tunnelState = uiState
        }

        if (Build.VERSION.SDK_INT >= 26) {
            initChannel()
        }

        service.startForeground(FOREGROUND_NOTIFICATION_ID, buildNotification())
    }

    fun onDestroy() {
        listenerId?.let { listener ->
            connectionProxy.onUiStateChange.unsubscribe(listener)
        }

        service.stopForeground(FOREGROUND_NOTIFICATION_ID)
    }

    private fun initChannel() {
        val channelName = service.getString(R.string.foreground_notification_channel_name)
        val importance = NotificationManager.IMPORTANCE_MIN
        val channel = NotificationChannel(CHANNEL_ID, channelName, importance).apply {
            description = service.getString(R.string.foreground_notification_channel_description)
            setShowBadge(true)
        }

        notificationManager.createNotificationChannel(channel)
    }

    private fun updateNotification() {
        if (!reconnecting || !showingReconnecting) {
            notificationManager.notify(FOREGROUND_NOTIFICATION_ID, buildNotification())
        }
    }

    private fun buildNotification(): Notification {
        val intent = Intent(service, MainActivity::class.java)
            .setFlags(Intent.FLAG_ACTIVITY_CLEAR_TOP or Intent.FLAG_ACTIVITY_SINGLE_TOP)
            .setAction(Intent.ACTION_MAIN)

        val pendingIntent =
            PendingIntent.getActivity(service, 1, intent, PendingIntent.FLAG_UPDATE_CURRENT)

        return NotificationCompat.Builder(service, CHANNEL_ID)
            .setSmallIcon(R.drawable.notification)
            .setColor(service.getColor(R.color.colorPrimary))
            .setContentTitle(service.getString(notificationText))
            .setContentIntent(pendingIntent)
            .build()
    }
}
