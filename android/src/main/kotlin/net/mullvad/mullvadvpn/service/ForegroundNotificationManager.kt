package net.mullvad.mullvadvpn.service

import android.app.KeyguardManager
import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.app.Service
import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.content.IntentFilter
import android.os.Build
import android.support.v4.app.NotificationCompat
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.model.TunnelState
import net.mullvad.mullvadvpn.ui.MainActivity
import net.mullvad.talpid.tunnel.ActionAfterDisconnect
import net.mullvad.talpid.util.EventNotifier

val CHANNEL_ID = "vpn_tunnel_status"
val FOREGROUND_NOTIFICATION_ID: Int = 1

class ForegroundNotificationManager(
    val service: MullvadVpnService,
    val serviceNotifier: EventNotifier<ServiceInstance?>,
    val keyguardManager: KeyguardManager
) {
    private val notificationManager =
        service.getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager

    private val badgeColor = service.resources.getColor(R.color.colorPrimary)

    private val deviceLockListener = object : BroadcastReceiver() {
        override fun onReceive(context: Context, intent: Intent) {
            val action = intent.action

            if (action == Intent.ACTION_USER_PRESENT || action == Intent.ACTION_SCREEN_OFF) {
                deviceIsUnlocked = !keyguardManager.isDeviceLocked
            }
        }
    }

    private var connectionProxy: ConnectionProxy? = null
        set(value) {
            if (field != value) {
                field?.onStateChange?.unsubscribe(this)

                value?.onStateChange?.subscribe(this) { state ->
                    tunnelState = state
                }

                field = value
            }
        }

    private var settingsListener: SettingsListener? = null
        set(value) {
            if (field != value) {
                field?.accountNumberNotifier?.unsubscribe(this)

                value?.accountNumberNotifier?.subscribe(this) { accountNumber ->
                    loggedIn = accountNumber != null
                }

                field = value
            }
        }

    private var onForeground = false
    private var reconnecting = false
    private var showingReconnecting = false

    private var tunnelState: TunnelState = TunnelState.Disconnected()
        set(value) {
            field = value

            reconnecting =
                (value is TunnelState.Disconnecting &&
                    value.actionAfterDisconnect == ActionAfterDisconnect.Reconnect) ||
                (value is TunnelState.Connecting && reconnecting)

            updateNotification()
        }

    private var deviceIsUnlocked = true
        set(value) {
            if (field != value) {
                field = value
                updateNotification()
            }
        }

    private var loggedIn = false
        set(value) {
            field = value
            updateNotification()
        }

    private val shouldBeOnForeground
        get() = lockedToForeground || !(tunnelState is TunnelState.Disconnected)

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
        }

    private val tunnelActionText: Int
        get() {
            val state = tunnelState

            return when (state) {
                is TunnelState.Disconnected -> R.string.connect
                is TunnelState.Connecting -> R.string.cancel
                is TunnelState.Connected -> R.string.disconnect
                is TunnelState.Disconnecting -> {
                    when (state.actionAfterDisconnect) {
                        ActionAfterDisconnect.Reconnect -> R.string.cancel
                        else -> R.string.connect
                    }
                }
                is TunnelState.Error -> {
                    if (state.errorState.isBlocking) {
                        R.string.disconnect
                    } else {
                        R.string.dismiss
                    }
                }
            }
        }

    private val tunnelActionKey: String
        get() {
            val state = tunnelState

            return when (state) {
                is TunnelState.Disconnected -> MullvadVpnService.KEY_CONNECT_ACTION
                is TunnelState.Connecting -> MullvadVpnService.KEY_DISCONNECT_ACTION
                is TunnelState.Connected -> MullvadVpnService.KEY_DISCONNECT_ACTION
                is TunnelState.Disconnecting -> {
                    when (state.actionAfterDisconnect) {
                        ActionAfterDisconnect.Reconnect -> MullvadVpnService.KEY_DISCONNECT_ACTION
                        else -> MullvadVpnService.KEY_CONNECT_ACTION
                    }
                }
                is TunnelState.Error -> MullvadVpnService.KEY_DISCONNECT_ACTION
            }
        }

    private val tunnelActionIcon: Int
        get() {
            if (tunnelActionKey == MullvadVpnService.KEY_CONNECT_ACTION) {
                return R.drawable.icon_notification_connect
            } else {
                return R.drawable.icon_notification_disconnect
            }
        }

    var lockedToForeground = false
        set(value) {
            field = value
            updateNotificationForegroundStatus()
        }

    init {
        if (Build.VERSION.SDK_INT >= 26) {
            initChannel()
        }

        serviceNotifier.subscribe(this) { newServiceInstance ->
            connectionProxy = newServiceInstance?.connectionProxy
            settingsListener = newServiceInstance?.settingsListener
        }

        service.apply {
            registerReceiver(deviceLockListener, IntentFilter().apply {
                addAction(Intent.ACTION_USER_PRESENT)
                addAction(Intent.ACTION_SCREEN_OFF)
            })
        }

        updateNotification()
    }

    fun onDestroy() {
        serviceNotifier.unsubscribe(this)
        connectionProxy = null
        settingsListener = null

        service.unregisterReceiver(deviceLockListener)

        notificationManager.cancel(FOREGROUND_NOTIFICATION_ID)
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

        updateNotificationForegroundStatus()
    }

    private fun updateNotificationForegroundStatus() {
        if (shouldBeOnForeground != onForeground) {
            if (shouldBeOnForeground) {
                service.startForeground(FOREGROUND_NOTIFICATION_ID, buildNotification())
                onForeground = true
            } else if (!shouldBeOnForeground) {
                if (Build.VERSION.SDK_INT >= 24) {
                    service.stopForeground(Service.STOP_FOREGROUND_DETACH)
                } else {
                    service.stopForeground(false)
                }

                onForeground = false
            }
        }
    }

    private fun buildNotification(): Notification {
        val intent = Intent(service, MainActivity::class.java)
            .setFlags(Intent.FLAG_ACTIVITY_CLEAR_TOP or Intent.FLAG_ACTIVITY_SINGLE_TOP)
            .setAction(Intent.ACTION_MAIN)

        val pendingIntent =
            PendingIntent.getActivity(service, 1, intent, PendingIntent.FLAG_UPDATE_CURRENT)

        val builder = NotificationCompat.Builder(service, CHANNEL_ID)
            .setSmallIcon(R.drawable.small_logo_black)
            .setColor(badgeColor)
            .setContentTitle(service.getString(notificationText))
            .setContentIntent(pendingIntent)

        if (loggedIn && deviceIsUnlocked) {
            builder.addAction(buildTunnelAction())
        }

        return builder.build()
    }

    private fun buildTunnelAction(): NotificationCompat.Action {
        val intent = Intent(tunnelActionKey).setPackage("net.mullvad.mullvadvpn")
        val flags = PendingIntent.FLAG_UPDATE_CURRENT

        val pendingIntent = if (Build.VERSION.SDK_INT >= 26) {
            PendingIntent.getForegroundService(service, 1, intent, flags)
        } else {
            PendingIntent.getService(service, 1, intent, flags)
        }

        val icon = tunnelActionIcon
        val label = service.getString(tunnelActionText)

        return NotificationCompat.Action(icon, label, pendingIntent)
    }
}
