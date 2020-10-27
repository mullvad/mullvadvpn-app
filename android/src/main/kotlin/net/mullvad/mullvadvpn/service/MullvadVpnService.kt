package net.mullvad.mullvadvpn.service

import android.app.KeyguardManager
import android.content.Context
import android.content.Intent
import android.net.VpnService
import android.os.Binder
import android.os.IBinder
import android.util.Log
import java.io.File
import kotlin.properties.Delegates.observable
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.model.Settings
import net.mullvad.mullvadvpn.service.notifications.AccountExpiryNotification
import net.mullvad.mullvadvpn.service.tunnelstate.TunnelStateUpdater
import net.mullvad.mullvadvpn.ui.MainActivity
import net.mullvad.talpid.TalpidVpnService
import net.mullvad.talpid.util.EventNotifier

private const val RELAYS_FILE = "relays.json"

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

    private val binder = LocalBinder()
    private val serviceNotifier = EventNotifier<ServiceInstance?>(null)

    private var isStopping = false
    private var shouldStop = false

    private var startDaemonJob: Job? = null

    private var instance by observable<ServiceInstance?>(null) { _, oldInstance, newInstance ->
        if (newInstance != oldInstance) {
            oldInstance?.onDestroy()

            accountExpiryNotification = newInstance?.let { instance ->
                AccountExpiryNotification(this, instance.daemon, instance.accountCache)
            }

            serviceNotifier.notify(newInstance)
        }
    }

    private var accountExpiryNotification
    by observable<AccountExpiryNotification?>(null) { _, oldNotification, _ ->
        oldNotification?.onDestroy()
    }

    private lateinit var keyguardManager: KeyguardManager
    private lateinit var notificationManager: ForegroundNotificationManager
    private lateinit var tunnelStateUpdater: TunnelStateUpdater

    private var pendingAction by observable<PendingAction?>(null) { _, _, action ->
        instance?.let { activeInstance ->
            handlePendingAction(
                activeInstance.connectionProxy,
                activeInstance.settingsListener.settings
            )
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

        keyguardManager = getSystemService(Context.KEYGUARD_SERVICE) as KeyguardManager
        notificationManager = ForegroundNotificationManager(this, serviceNotifier, keyguardManager)
        tunnelStateUpdater = TunnelStateUpdater(this, serviceNotifier)

        setUp()
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

        if (shouldStop && !quitCommand) {
            shouldStop = false

            if (isStopping) {
                restart()
                isStopping = false
            }
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

        if (isStopping) {
            restart()
            isStopping = false
        }
    }

    override fun onRevoke() {
        pendingAction = PendingAction.Disconnect
    }

    override fun onUnbind(intent: Intent): Boolean {
        Log.d(TAG, "Closed all connections to service")
        isBound = false

        if (shouldStop) {
            stop()
        }

        return true
    }

    override fun onDestroy() {
        Log.d(TAG, "Service has stopped")
        tearDown()
        notificationManager.onDestroy()
        super.onDestroy()
    }

    inner class LocalBinder : Binder() {
        val serviceNotifier
            get() = this@MullvadVpnService.serviceNotifier

        var isUiVisible
            get() = this@MullvadVpnService.isUiVisible
            set(value) { this@MullvadVpnService.isUiVisible = value }
    }

    private fun setUp() {
        startDaemonJob?.cancel()
        startDaemonJob = startDaemon()
    }

    private fun startDaemon() = GlobalScope.launch(Dispatchers.Default) {
        Log.d(TAG, "Starting daemon")
        prepareFiles()

        val daemon = MullvadDaemon(this@MullvadVpnService).apply {
            onDaemonStopped = {
                Log.d(TAG, "Daemon has stopped")
                instance = null

                if (!isStopping) {
                    restart()
                }
            }
        }

        val settings = daemon.getSettings()

        if (settings != null) {
            setUpInstance(daemon, settings)
        } else {
            restart()
        }
    }

    private fun prepareFiles() {
        FileMigrator(File("/data/data/net.mullvad.mullvadvpn"), filesDir).apply {
            migrate(RELAYS_FILE)
            migrate("settings.json")
            migrate("daemon.log")
            migrate("daemon.old.log")
            migrate("wireguard.log")
            migrate("wireguard.old.log")
        }

        val shouldOverwriteRelayList =
            lastUpdatedTime() > File(filesDir, RELAYS_FILE).lastModified()

        FileResourceExtractor(this).apply {
            extract(RELAYS_FILE, shouldOverwriteRelayList)
        }
    }

    private fun setUpInstance(daemon: MullvadDaemon, settings: Settings) {
        val settingsListener = SettingsListener(daemon, settings)
        val connectionProxy = ConnectionProxy(this, daemon)

        val splitTunneling = SplitTunneling(this).apply {
            onChange = { excludedApps ->
                disallowedApps = excludedApps
                markTunAsStale()
                connectionProxy.reconnect()
            }
        }

        handlePendingAction(connectionProxy, settings)

        instance = ServiceInstance(
            daemon,
            connectionProxy,
            connectivityListener,
            settingsListener,
            splitTunneling
        )
    }

    private fun stop() {
        Log.d(TAG, "Stopping service")
        isStopping = true
        shouldStop = true
        stopDaemon()
        stopSelf()
    }

    private fun stopDaemon() {
        Log.d(TAG, "Stopping daemon")
        startDaemonJob?.cancel()
        instance?.daemon?.shutdown()
    }

    private fun tearDown() {
        stopDaemon()
    }

    private fun restart() {
        Log.d(TAG, "Restarting service")
        tearDown()
        setUp()
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

    private fun lastUpdatedTime(): Long {
        return packageManager.getPackageInfo(packageName, 0).lastUpdateTime
    }
}
