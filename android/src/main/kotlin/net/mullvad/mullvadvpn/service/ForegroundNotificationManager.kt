package net.mullvad.mullvadvpn.service

import android.app.KeyguardManager
import android.app.Service
import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.content.IntentFilter
import android.os.Build
import kotlin.properties.Delegates.observable
import kotlinx.coroutines.Dispatchers
import net.mullvad.mullvadvpn.model.TunnelState
import net.mullvad.mullvadvpn.service.ForegroundNotificationManager.UpdaterMessage
import net.mullvad.mullvadvpn.service.endpoint.Actor
import net.mullvad.mullvadvpn.service.endpoint.ConnectionProxy
import net.mullvad.mullvadvpn.service.notifications.TunnelStateNotification
import net.mullvad.talpid.util.autoSubscribable

class ForegroundNotificationManager(
    val service: MullvadVpnService,
    val connectionProxy: ConnectionProxy,
    val keyguardManager: KeyguardManager
) : Actor<UpdaterMessage>(Dispatchers.Main) {
    sealed class UpdaterMessage {
        class UpdateNotification : UpdaterMessage()
        class UpdateAction : UpdaterMessage()
        class NewTunnelState(val newState: TunnelState) : UpdaterMessage()
    }

    private val tunnelStateNotification = TunnelStateNotification(service)
    private val deviceLockListener = object : BroadcastReceiver() {
        override fun onReceive(context: Context, intent: Intent) {
            val action = intent.action

            if (action == Intent.ACTION_USER_PRESENT || action == Intent.ACTION_SCREEN_OFF) {
                deviceIsUnlocked = !keyguardManager.isDeviceLocked
            }
        }
    }

    private var deviceIsUnlocked by observable(!keyguardManager.isDeviceLocked) { _, _, _ ->
        sendBlocking(UpdaterMessage.UpdateAction())
    }

    private var loggedIn by observable(false) { _, _, _ ->
        sendBlocking(UpdaterMessage.UpdateAction())
    }

    private val tunnelState
        get() = connectionProxy.onStateChange.latestEvent

    private val shouldBeOnForeground
        get() = lockedToForeground || !(tunnelState is TunnelState.Disconnected)

    var accountNumberEvents by autoSubscribable<String?>(this, null) { accountNumber ->
        loggedIn = accountNumber != null
    }

    var onForeground = false
        private set

    var lockedToForeground by observable(false) { _, _, _ ->
        sendBlocking(UpdaterMessage.UpdateNotification())
    }

    init {
        connectionProxy.onStateChange.subscribe(this) { newState ->
            sendBlocking(UpdaterMessage.NewTunnelState(newState))
        }

        service.registerReceiver(
            deviceLockListener,
            IntentFilter().apply {
                addAction(Intent.ACTION_USER_PRESENT)
                addAction(Intent.ACTION_SCREEN_OFF)
            }
        )

        sendBlocking(UpdaterMessage.UpdateNotification())
    }

    fun onDestroy() {
        accountNumberEvents = null

        connectionProxy.onStateChange.unsubscribe(this)
        service.unregisterReceiver(deviceLockListener)

        closeActor()

        tunnelStateNotification.visible = false
    }

    fun acknowledgeStartForegroundService() {
        // When sending start commands to the service, it is necessary to request the service to be
        // on the foreground. With such request, when the service is started it must be placed on
        // the foreground with a call to startForeground before a timeout expires, otherwise Android
        // kills the app.
        showOnForeground()

        // Restore the notification to its correct state.
        updateNotification()
    }

    override suspend fun onNewCommand(command: UpdaterMessage) = when (command) {
        is UpdaterMessage.UpdateNotification -> updateNotification()
        is UpdaterMessage.UpdateAction -> updateNotificationAction()
        is UpdaterMessage.NewTunnelState -> {
            tunnelStateNotification.tunnelState = command.newState
            updateNotification()
        }
    }

    private fun showOnForeground() {
        service.startForeground(
            TunnelStateNotification.NOTIFICATION_ID,
            tunnelStateNotification.build()
        )

        onForeground = true
    }

    private fun updateNotification() {
        if (shouldBeOnForeground != onForeground) {
            if (shouldBeOnForeground) {
                showOnForeground()
            } else {
                if (Build.VERSION.SDK_INT >= 24) {
                    service.stopForeground(Service.STOP_FOREGROUND_DETACH)
                } else {
                    service.stopForeground(false)
                }

                onForeground = false
            }
        }
    }

    private fun updateNotificationAction() {
        tunnelStateNotification.showAction = loggedIn && deviceIsUnlocked
    }
}
