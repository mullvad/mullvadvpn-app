package net.mullvad.mullvadvpn.service

import android.app.KeyguardManager
import android.content.Context
import android.content.Intent
import android.net.VpnService
import android.os.Binder
import android.os.IBinder
import android.os.Looper
import android.util.Log
import kotlin.properties.Delegates.observable
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.model.Settings
import net.mullvad.mullvadvpn.service.endpoint.ServiceEndpoint
import net.mullvad.mullvadvpn.service.notifications.AccountExpiryNotification
import net.mullvad.mullvadvpn.service.tunnelstate.TunnelStateUpdater
import net.mullvad.mullvadvpn.ui.MainActivity
import net.mullvad.talpid.TalpidVpnService
import net.mullvad.talpid.util.EventNotifier

class MullvadVpnService : TalpidVpnService() {
    companion object {
        private val TAG = "mullvad"

        val KEY_CONNECT_ACTION = "net.mullvad.mullvadvpn.connect_action"
        val KEY_DISCONNECT_ACTION = "net.mullvad.mullvadvpn.disconnect_action"
        val KEY_QUIT_ACTION = "net.mullvad.mullvadvpn.quit_action"

        init {
            System.loadLibrary("mullvad_jni")
        }
    }

    private enum class PendingAction {
        Connect,
        Disconnect,
    }

    private enum class State {
        Running,
        Stopping,
        Stopped,
    }

    private val binder = LocalBinder()
    private val serviceNotifier = EventNotifier<ServiceInstance?>(null)

    private var state = State.Running

    private var setUpDaemonJob: Job? = null

    private var instance by observable<ServiceInstance?>(null) { _, oldInstance, newInstance ->
        if (newInstance != oldInstance) {
            oldInstance?.onDestroy()

            accountExpiryNotification = newInstance?.let { instance ->
                AccountExpiryNotification(this, instance.daemon, endpoint.accountCache)
            }

            serviceNotifier.notify(newInstance)
        }
    }

    private var accountExpiryNotification by observable<AccountExpiryNotification?>(null) {
        _, oldNotification, _ ->
        oldNotification?.onDestroy()
    }

    private lateinit var daemonInstance: DaemonInstance
    private lateinit var endpoint: ServiceEndpoint
    private lateinit var keyguardManager: KeyguardManager
    private lateinit var notificationManager: ForegroundNotificationManager
    private lateinit var tunnelStateUpdater: TunnelStateUpdater

    private var pendingAction by observable<PendingAction?>(null) { _, _, _ ->
        val connectionProxy = instance?.connectionProxy

        // The service instance awaits the split tunneling initialization, which also starts the
        // endpoint. So if the instance is not null, the endpoint has certainly been initialized.
        if (connectionProxy != null) {
            endpoint.settingsListener.settings?.let { settings ->
                handlePendingAction(connectionProxy, settings)
            }
        }
    }

    private var isBound: Boolean by observable(false) { _, _, isBound ->
        notificationManager.lockedToForeground = isUiVisible or isBound
    }

    private var isUiVisible: Boolean by observable(false) { _, _, isUiVisible ->
        notificationManager.lockedToForeground = isUiVisible or isBound
    }

    override fun onCreate() {
        super.onCreate()
        Log.d(TAG, "Initializing service")

        daemonInstance = DaemonInstance(this)
        keyguardManager = getSystemService(Context.KEYGUARD_SERVICE) as KeyguardManager
        tunnelStateUpdater = TunnelStateUpdater(this, serviceNotifier)

        endpoint = ServiceEndpoint(
            this,
            Looper.getMainLooper(),
            daemonInstance.intermittentDaemon,
            connectivityListener
        )

        notificationManager =
            ForegroundNotificationManager(this, serviceNotifier, keyguardManager).apply {
                acknowledgeStartForegroundService()
                accountNumberEvents = endpoint.settingsListener.accountNumberNotifier
            }

        daemonInstance.apply {
            intermittentDaemon.registerListener(this@MullvadVpnService) { daemon ->
                handleDaemonInstance(daemon)
            }

            start()
        }
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        Log.d(TAG, "Starting service")
        val startResult = super.onStartCommand(intent, flags, startId)
        var quitCommand = false

        notificationManager.acknowledgeStartForegroundService()

        if (!keyguardManager.isDeviceLocked) {
            val action = intent?.action

            if (action == VpnService.SERVICE_INTERFACE || action == KEY_CONNECT_ACTION) {
                pendingAction = PendingAction.Connect
            } else if (action == KEY_DISCONNECT_ACTION) {
                pendingAction = PendingAction.Disconnect
            } else if (action == KEY_QUIT_ACTION && !notificationManager.onForeground) {
                quitCommand = true
                stop()
            }
        }

        if (state == State.Stopping && !quitCommand) {
            restart()
        }

        return startResult
    }

    override fun onBind(intent: Intent): IBinder {
        Log.d(TAG, "New connection to service")
        isBound = true

        return super.onBind(intent) ?: binder
    }

    override fun onRebind(intent: Intent) {
        Log.d(TAG, "Connection to service restored")
        isBound = true

        if (state == State.Stopping) {
            restart()
        }
    }

    override fun onRevoke() {
        pendingAction = PendingAction.Disconnect
    }

    override fun onUnbind(intent: Intent): Boolean {
        Log.d(TAG, "Closed all connections to service")
        isBound = false

        if (state != State.Running) {
            stop()
        }

        return true
    }

    override fun onDestroy() {
        Log.d(TAG, "Service has stopped")
        state = State.Stopped
        notificationManager.onDestroy()
        daemonInstance.onDestroy()
        instance = null
        super.onDestroy()
    }

    inner class LocalBinder : Binder() {
        val serviceNotifier
            get() = this@MullvadVpnService.serviceNotifier

        var isUiVisible
            get() = this@MullvadVpnService.isUiVisible
            set(value) { this@MullvadVpnService.isUiVisible = value }
    }

    private fun handleDaemonInstance(daemon: MullvadDaemon?) {
        setUpDaemonJob?.cancel()

        if (daemon != null) {
            setUpDaemonJob = setUpDaemon(daemon)
        } else {
            Log.d(TAG, "Daemon has stopped")
            instance = null

            if (state == State.Running) {
                restart()
            }
        }
    }

    private fun setUpDaemon(daemon: MullvadDaemon) = GlobalScope.launch(Dispatchers.Main) {
        if (state != State.Stopped) {
            val settings = daemon.getSettings()

            if (settings != null) {
                setUpInstance(daemon, settings)
            } else {
                restart()
            }
        }
    }

    private suspend fun setUpInstance(daemon: MullvadDaemon, settings: Settings) {
        val connectionProxy = ConnectionProxy(this, daemon)
        val customDns = CustomDns(daemon, endpoint.settingsListener)

        endpoint.splitTunneling.onChange.subscribe(this@MullvadVpnService) { excludedApps ->
            disallowedApps = excludedApps
            markTunAsStale()
            connectionProxy.reconnect()
        }

        handlePendingAction(connectionProxy, settings)

        endpoint.locationInfoCache.stateEvents = connectionProxy.onStateChange

        if (state == State.Running) {
            instance = ServiceInstance(
                endpoint.messenger,
                daemon,
                daemonInstance.intermittentDaemon,
                connectionProxy,
                customDns
            )
        }
    }

    private fun stop() {
        Log.d(TAG, "Stopping service")
        state = State.Stopping
        daemonInstance.stop()
        stopSelf()
    }

    private fun restart() {
        if (state != State.Stopped) {
            Log.d(TAG, "Restarting service")

            state = State.Running

            daemonInstance.apply {
                stop()
                start()
            }
        } else {
            Log.d(TAG, "Ignoring restart because onDestroy has executed")
        }
    }

    private fun handlePendingAction(connectionProxy: ConnectionProxy, settings: Settings) {
        when (pendingAction) {
            PendingAction.Connect -> {
                if (settings.accountToken != null) {
                    connectionProxy.connect()
                } else {
                    openUi()
                }
            }
            PendingAction.Disconnect -> connectionProxy.disconnect()
            null -> return
        }

        pendingAction = null
    }

    private fun openUi() {
        val intent = Intent(this, MainActivity::class.java).apply {
            addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
            addFlags(Intent.FLAG_ACTIVITY_CLEAR_TOP)
        }

        startActivity(intent)
    }
}
