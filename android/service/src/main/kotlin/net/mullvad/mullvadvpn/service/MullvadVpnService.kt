package net.mullvad.mullvadvpn.service

import android.annotation.SuppressLint
import android.app.KeyguardManager
import android.content.Context
import android.content.Intent
import android.os.IBinder
import android.os.Looper
import android.util.Log
import kotlin.properties.Delegates.observable
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.lib.common.constant.KEY_CONNECT_ACTION
import net.mullvad.mullvadvpn.lib.common.constant.KEY_DISCONNECT_ACTION
import net.mullvad.mullvadvpn.lib.common.constant.KEY_QUIT_ACTION
import net.mullvad.mullvadvpn.lib.common.constant.MAIN_ACTIVITY_CLASS
import net.mullvad.mullvadvpn.lib.endpoint.ApiEndpointConfiguration
import net.mullvad.mullvadvpn.lib.endpoint.getApiEndpointConfigurationExtras
import net.mullvad.mullvadvpn.model.Settings
import net.mullvad.mullvadvpn.model.TunnelState
import net.mullvad.mullvadvpn.service.di.apiEndpointModule
import net.mullvad.mullvadvpn.service.di.vpnServiceModule
import net.mullvad.mullvadvpn.service.endpoint.ServiceEndpoint
import net.mullvad.mullvadvpn.service.notifications.AccountExpiryNotification
import net.mullvad.talpid.TalpidVpnService
import org.koin.android.ext.android.get
import org.koin.core.context.loadKoinModules

class MullvadVpnService : TalpidVpnService() {

    private enum class PendingAction {
        Connect,
        Disconnect,
    }

    private enum class State {
        Running,
        Stopping,
        Stopped,
    }

    private val connectionProxy
        get() = endpoint.connectionProxy

    private var state = State.Running

    private var setUpDaemonJob: Job? = null

    private lateinit var accountExpiryNotification: AccountExpiryNotification
    private lateinit var daemonInstance: DaemonInstance
    private lateinit var endpoint: ServiceEndpoint
    private lateinit var keyguardManager: KeyguardManager
    private lateinit var notificationManager: ForegroundNotificationManager

    private var pendingAction by
        observable<PendingAction?>(null) { _, _, _ ->
            endpoint.settingsListener.settings?.let { settings -> handlePendingAction(settings) }
        }

    private lateinit var apiEndpointConfiguration: ApiEndpointConfiguration

    // Suppressing since the tunnel state pref should be writted immediately.
    @SuppressLint("ApplySharedPref")
    override fun onCreate() {
        super.onCreate()
        Log.d(TAG, "Initializing service")

        loadKoinModules(listOf(vpnServiceModule, apiEndpointModule))

        daemonInstance = DaemonInstance(this)
        keyguardManager = getSystemService(Context.KEYGUARD_SERVICE) as KeyguardManager

        endpoint =
            ServiceEndpoint(
                Looper.getMainLooper(),
                daemonInstance.intermittentDaemon,
                connectivityListener,
                this
            )

        endpoint.splitTunneling.onChange.subscribe(this@MullvadVpnService) { excludedApps ->
            disallowedApps = excludedApps
            markTunAsStale()
            connectionProxy.reconnect()
        }

        notificationManager =
            ForegroundNotificationManager(this, connectionProxy, daemonInstance.intermittentDaemon)

        accountExpiryNotification =
            AccountExpiryNotification(
                this,
                daemonInstance.intermittentDaemon,
                endpoint.accountCache
            )

        // Remove any leftover tunnel state persistence data
        getSharedPreferences("tunnel_state", MODE_PRIVATE).edit().clear().commit()
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        Log.d(TAG, "Starting service")

        val intentProvidedConfiguration =
            if (BuildConfig.DEBUG) {
                intent?.getApiEndpointConfigurationExtras()
            } else {
                null
            }

        apiEndpointConfiguration = intentProvidedConfiguration ?: get()

        daemonInstance.apply {
            intermittentDaemon.registerListener(this@MullvadVpnService) { daemon ->
                handleDaemonInstance(daemon)
            }

            start(apiEndpointConfiguration)
        }

        val startResult = super.onStartCommand(intent, flags, startId)
        var quitCommand = false

        // Always promote to foreground if connect/disconnect actions are provided to mitigate cases
        // where the service would potentially otherwise be too slow running `startForeground`.
        if (intent?.action == KEY_CONNECT_ACTION || intent?.action == KEY_DISCONNECT_ACTION) {
            notificationManager.showOnForeground()
        }

        notificationManager.updateNotification()

        if (!keyguardManager.isDeviceLocked) {
            val action = intent?.action

            if (action == SERVICE_INTERFACE || action == KEY_CONNECT_ACTION) {
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
        return super.onBind(intent) ?: endpoint.messenger.binder
    }

    override fun onRebind(intent: Intent) {
        Log.d(TAG, "Connection to service restored")
        if (state == State.Stopping) {
            restart()
        }
    }

    override fun onRevoke() {
        pendingAction = PendingAction.Disconnect
    }

    override fun onUnbind(intent: Intent): Boolean {
        Log.d(TAG, "Closed all connections to service")

        if (state != State.Running) {
            stop()
        }

        return true
    }

    override fun onDestroy() {
        Log.d(TAG, "Service has stopped")
        state = State.Stopped
        accountExpiryNotification.onDestroy()
        notificationManager.onDestroy()
        daemonInstance.onDestroy()
        super.onDestroy()
    }

    override fun onTaskRemoved(rootIntent: Intent?) {
        connectionProxy.onStateChange.latestEvent.let { tunnelState ->
            Log.d(TAG, "Task removed (tunnelState=$tunnelState)")
            if (tunnelState is TunnelState.Disconnected) {
                notificationManager.cancelNotification()
                stop()
            }
        }
    }

    private fun handleDaemonInstance(daemon: MullvadDaemon?) {
        setUpDaemonJob?.cancel()

        if (daemon != null) {
            setUpDaemonJob = setUpDaemon(daemon)
        } else {
            Log.d(TAG, "Daemon has stopped")

            if (state == State.Running) {
                restart()
            }
        }
    }

    private fun setUpDaemon(daemon: MullvadDaemon) =
        GlobalScope.launch(Dispatchers.Main) {
            if (state != State.Stopped) {
                val settings = daemon.getSettings()

                if (settings != null) {
                    handlePendingAction(settings)
                } else {
                    restart()
                }
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
                start(apiEndpointConfiguration)
            }
        } else {
            Log.d(TAG, "Ignoring restart because onDestroy has executed")
        }
    }

    private fun handlePendingAction(settings: Settings) {
        when (pendingAction) {
            PendingAction.Connect -> {
                if (settings != null) {
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
        val intent =
            Intent().apply {
                setClassName(applicationContext.packageName, MAIN_ACTIVITY_CLASS)
                addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
                addFlags(Intent.FLAG_ACTIVITY_CLEAR_TOP)
            }

        startActivity(intent)
    }

    companion object {
        private val TAG = "mullvad"

        init {
            System.loadLibrary("mullvad_jni")
        }
    }
}
