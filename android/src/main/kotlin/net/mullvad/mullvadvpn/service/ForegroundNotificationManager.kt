package net.mullvad.mullvadvpn.service

import android.app.KeyguardManager
import android.app.Service
import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.content.IntentFilter
import android.os.Build
import kotlin.properties.Delegates.observable
import net.mullvad.mullvadvpn.model.TunnelState
import net.mullvad.mullvadvpn.service.notifications.TunnelStateNotification
import net.mullvad.talpid.util.EventNotifier
import net.mullvad.talpid.util.autoSubscribable

class ForegroundNotificationManager(
    val service: MullvadVpnService,
    val serviceNotifier: EventNotifier<ServiceInstance?>,
    val keyguardManager: KeyguardManager
) {
    private val tunnelStateNotification = TunnelStateNotification(service)

    private val deviceLockListener = object : BroadcastReceiver() {
        override fun onReceive(context: Context, intent: Intent) {
            val action = intent.action

            if (action == Intent.ACTION_USER_PRESENT || action == Intent.ACTION_SCREEN_OFF) {
                deviceIsUnlocked = !keyguardManager.isDeviceLocked
            }
        }
    }

    private var accountNumberEvents by autoSubscribable<String?>(this, null) { accountNumber ->
        loggedIn = accountNumber != null
    }

    private var tunnelStateEvents
    by autoSubscribable<TunnelState>(this, TunnelState.Disconnected()) { newState ->
        tunnelStateNotification.tunnelState = newState
        updateNotification()
    }

    private var deviceIsUnlocked by observable(!keyguardManager.isDeviceLocked) { _, _, _ ->
        updateNotificationAction()
    }

    private var loggedIn by observable(false) { _, _, _ -> updateNotificationAction() }

    private val tunnelState
        get() = tunnelStateEvents?.latestEvent ?: TunnelState.Disconnected()

    private val shouldBeOnForeground
        get() = lockedToForeground || !(tunnelState is TunnelState.Disconnected)

    var onForeground = false
        private set

    var lockedToForeground by observable(false) { _, _, _ -> updateNotification() }

    init {
        serviceNotifier.subscribe(this) { newServiceInstance ->
            accountNumberEvents = newServiceInstance?.settingsListener?.accountNumberNotifier
            tunnelStateEvents = newServiceInstance?.connectionProxy?.onStateChange
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

        updateNotification()
    }

    fun onDestroy() {
        serviceNotifier.unsubscribe(this)

        accountNumberEvents = null
        tunnelStateEvents = null

        service.unregisterReceiver(deviceLockListener)

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

    private fun updateNotificationAction() {
        tunnelStateNotification.showAction = loggedIn && deviceIsUnlocked
    }
}
