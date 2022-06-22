package net.mullvad.mullvadvpn.service

import android.app.Service
import kotlin.properties.Delegates.observable
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.channels.actor
import kotlinx.coroutines.channels.sendBlocking
import kotlinx.coroutines.flow.collect
import net.mullvad.mullvadvpn.model.DeviceState
import net.mullvad.mullvadvpn.model.TunnelState
import net.mullvad.mullvadvpn.service.endpoint.ConnectionProxy
import net.mullvad.mullvadvpn.service.notifications.TunnelStateNotification
import net.mullvad.mullvadvpn.util.Intermittent
import net.mullvad.mullvadvpn.util.JobTracker

class ForegroundNotificationManager(
    val service: MullvadVpnService,
    val connectionProxy: ConnectionProxy,
    val intermittentDaemon: Intermittent<MullvadDaemon>
) {
    private sealed class UpdaterMessage {
        class UpdateNotification : UpdaterMessage()
        class UpdateAction : UpdaterMessage()
        class NewTunnelState(val newState: TunnelState) : UpdaterMessage()
    }

    private val jobTracker = JobTracker()
    private val updater = runUpdater()

    private val tunnelStateNotification = TunnelStateNotification(service)

    private var loggedIn by observable(false) { _, _, _ ->
        updater.sendBlocking(UpdaterMessage.UpdateAction())
    }

    private val tunnelState
        get() = connectionProxy.onStateChange.latestEvent

    private val shouldBeOnForeground
        get() = lockedToForeground || !(tunnelState is TunnelState.Disconnected)

    var onForeground = false
        private set

    var lockedToForeground by observable(false) { _, _, _ ->
        updater.sendBlocking(UpdaterMessage.UpdateNotification())
    }

    init {
        connectionProxy.onStateChange.subscribe(this) { newState ->
            updater.sendBlocking(UpdaterMessage.NewTunnelState(newState))
        }

        intermittentDaemon.registerListener(this) { daemon ->
            jobTracker.newBackgroundJob("notificationLoggedInJob") {
                daemon?.deviceStateUpdates?.collect { deviceState ->
                    loggedIn = deviceState is DeviceState.LoggedIn
                }
            }
        }

        updater.sendBlocking(UpdaterMessage.UpdateNotification())
    }

    fun onDestroy() {
        jobTracker.cancelAllJobs()
        intermittentDaemon.unregisterListener(this)
        connectionProxy.onStateChange.unsubscribe(this)
        updater.close()
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

    fun updateNotification() {
        if (shouldBeOnForeground != onForeground) {
            if (shouldBeOnForeground) {
                showOnForeground()
            } else {
                service.stopForeground(Service.STOP_FOREGROUND_DETACH)
                onForeground = false
            }
        }
    }

    fun cancelNotification() {
        tunnelStateNotification.visible = false
    }

    private fun updateNotificationAction() {
        tunnelStateNotification.showAction = loggedIn
    }
}
