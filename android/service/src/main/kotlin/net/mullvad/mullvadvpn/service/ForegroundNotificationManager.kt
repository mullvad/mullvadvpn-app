package net.mullvad.mullvadvpn.service

import android.content.pm.ServiceInfo.FOREGROUND_SERVICE_TYPE_SYSTEM_EXEMPTED
import android.net.VpnService
import android.os.Build
import kotlin.properties.Delegates.observable
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.channels.actor
import kotlinx.coroutines.channels.trySendBlocking
import net.mullvad.mullvadvpn.lib.common.util.JobTracker
import net.mullvad.mullvadvpn.model.TunnelState
import net.mullvad.mullvadvpn.service.notifications.TunnelStateNotification

class ForegroundNotificationManager(
    val vpnService: VpnService,
) {
    private sealed class UpdaterMessage {
        class UpdateNotification : UpdaterMessage()

        class UpdateAction : UpdaterMessage()

        class NewTunnelState(val newState: TunnelState) : UpdaterMessage()
    }

    private val jobTracker = JobTracker()
    private val updater = runUpdater()

    private val tunnelStateNotification = TunnelStateNotification(vpnService)

    private var loggedIn by
        observable(false) { _, _, _ -> updater.trySendBlocking(UpdaterMessage.UpdateAction()) }

    //    private val tunnelState
    //        get() = connectionProxy.onStateChange.latestEvent

    //    private val shouldBeOnForeground
    //        get() = lockedToForeground || !(tunnelState is TunnelState.Disconnected)

    var onForeground = false
        private set

    init {
        //        connectionProxy.onStateChange.subscribe(this) { newState ->
        //            updater.trySendBlocking(UpdaterMessage.NewTunnelState(newState))
        //        }
        //
        //        intermittentDaemon.registerListener(this) { daemon ->
        //            jobTracker.newBackgroundJob("notificationLoggedInJob") {
        //                daemon
        //                    ?.deviceStateUpdates
        //                    ?.onStart { daemon.getAndEmitDeviceState()?.let { emit(it) } }
        //                    ?.collect { deviceState -> loggedIn = deviceState is
        // DeviceState.LoggedIn }
        //            }
        //        }

        updater.trySendBlocking(UpdaterMessage.UpdateNotification())
    }

    fun onDestroy() {
        jobTracker.cancelAllJobs()
        //        connectionProxy.onStateChange.unsubscribe(this)
        updater.close()
    }

    private fun runUpdater() =
        GlobalScope.actor<UpdaterMessage>(Dispatchers.Main, Channel.UNLIMITED) {
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

    fun showOnForeground() {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.UPSIDE_DOWN_CAKE) {
            if (VpnService.prepare(vpnService) == null) {
                vpnService.startForeground(
                    TunnelStateNotification.NOTIFICATION_ID,
                    tunnelStateNotification.build(),
                    FOREGROUND_SERVICE_TYPE_SYSTEM_EXEMPTED
                )
            } else {
                return
            }
        } else {
            vpnService.startForeground(
                TunnelStateNotification.NOTIFICATION_ID,
                tunnelStateNotification.build(),
            )
        }
        onForeground = true
    }

    fun updateNotification() {
        //        if (shouldBeOnForeground != onForeground) {
        //            if (shouldBeOnForeground) {
        //                showOnForeground()
        //            } else {
        //                service.stopForeground(Service.STOP_FOREGROUND_DETACH)
        //                onForeground = false
        //            }
        //        }
    }

    fun cancelNotification() {
        tunnelStateNotification.visible = false
    }

    private fun updateNotificationAction() {
        tunnelStateNotification.showAction = loggedIn
    }
}
