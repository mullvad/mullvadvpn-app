package net.mullvad.mullvadvpn.service

import android.app.KeyguardManager
import android.app.Service
import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.content.IntentFilter
import kotlin.properties.Delegates.observable
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.channels.actor
import kotlinx.coroutines.channels.sendBlocking
import kotlinx.coroutines.flow.collect
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.model.TunnelState
import net.mullvad.mullvadvpn.service.endpoint.ConnectionProxy
import net.mullvad.mullvadvpn.service.endpoint.ForegroundRequestHandler
import net.mullvad.mullvadvpn.service.notifications.TunnelStateNotification
import net.mullvad.talpid.util.autoSubscribable

class ForegroundNotificationManager(
    val service: MullvadVpnService,
    val connectionProxy: ConnectionProxy,
    val keyguardManager: KeyguardManager,
    val foregroundRequestHandler: ForegroundRequestHandler,
) {
    private sealed class UpdaterMessage {
        class UpdateNotification : UpdaterMessage()
        class UpdateAction : UpdaterMessage()
        class NewTunnelState(val newState: TunnelState) : UpdaterMessage()
    }

    private val updater = runUpdater()

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
        updater.sendBlocking(UpdaterMessage.UpdateAction())
    }

    private var loggedIn by observable(false) { _, _, _ ->
        updater.sendBlocking(UpdaterMessage.UpdateAction())
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
        updater.sendBlocking(UpdaterMessage.UpdateNotification())
    }

    var forcedForegroundRequestJob: Job

    init {
        connectionProxy.onStateChange.subscribe(this) { newState ->
            updater.sendBlocking(UpdaterMessage.NewTunnelState(newState))
        }

        service.apply {
            registerReceiver(
                deviceLockListener,
                IntentFilter().apply {
                    addAction(Intent.ACTION_USER_PRESENT)
                    addAction(Intent.ACTION_SCREEN_OFF)
                }
            )
        }

        updater.sendBlocking(UpdaterMessage.UpdateNotification())

        forcedForegroundRequestJob = GlobalScope.launch(Dispatchers.Main) {
            foregroundRequestHandler.foregroundRequests().collect {
                lockedToForeground = it
            }
        }
    }

    fun onDestroy() {
        accountNumberEvents = null

        connectionProxy.onStateChange.unsubscribe(this)
        service.unregisterReceiver(deviceLockListener)

        updater.close()

        tunnelStateNotification.visible = false

        forcedForegroundRequestJob.cancel()
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

    private fun runUpdater() = GlobalScope.actor<UpdaterMessage>(
        Dispatchers.Main,
        Channel.UNLIMITED
    ) {
        for (message in channel) {
            when (message) {
                is UpdaterMessage.UpdateNotification -> updateNotification()
                is UpdaterMessage.UpdateAction -> updateNotificationAction()
                is UpdaterMessage.NewTunnelState -> {
                    tunnelStateNotification.tunnelState = message.newState
                    updateNotification()
                }
            }
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
                service.stopForeground(Service.STOP_FOREGROUND_DETACH)
                onForeground = false
            }
        }
    }

    private fun updateNotificationAction() {
        tunnelStateNotification.showAction = loggedIn && deviceIsUnlocked
    }
}
