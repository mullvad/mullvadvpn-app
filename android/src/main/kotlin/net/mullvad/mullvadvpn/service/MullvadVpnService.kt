package net.mullvad.mullvadvpn.service

import android.content.Intent
import android.net.VpnService
import android.os.Binder
import android.os.IBinder
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.dataproxy.ConnectionProxy
import net.mullvad.mullvadvpn.service.tunnelstate.TunnelStateUpdater
import net.mullvad.mullvadvpn.ui.MainActivity
import net.mullvad.talpid.TalpidVpnService
import net.mullvad.talpid.util.EventNotifier

private const val API_ROOT_CA_FILE = "api_root_ca.pem"
private const val API_ROOT_CA_PATH = "/data/data/net.mullvad.mullvadvpn/api_root_ca.pem"

private const val RELAYS_FILE = "relays.json"
private const val RELAYS_PATH = "/data/data/net.mullvad.mullvadvpn/relays.json"

class MullvadVpnService : TalpidVpnService() {
    private enum class PendingAction {
        Connect,
        Disconnect,
    }

    private val binder = LocalBinder()
    private val serviceNotifier = EventNotifier<ServiceInstance?>(null)

    private var isStopping = false

    private var connectionProxy: ConnectionProxy? = null
    private var daemon: MullvadDaemon? = null
    private var startDaemonJob: Job? = null

    private lateinit var notificationManager: ForegroundNotificationManager
    private lateinit var tunnelStateUpdater: TunnelStateUpdater

    private var pendingAction: PendingAction? = null
        set(value) {
            field = value

            connectionProxy?.let { activeConnectionProxy ->
                when (value) {
                    PendingAction.Connect -> activeConnectionProxy.connect()
                    PendingAction.Disconnect -> activeConnectionProxy.disconnect()
                    null -> {}
                }

                field = null
            }
        }

    private var bindCount = 0
        set(value) {
            field = value
            isBound = bindCount != 0
        }

    private var isBound = false
        set(value) {
            field = value
            notificationManager.lockedToForeground = value
        }

    private var loggedIn = false
        set(value) {
            field = value
            notificationManager.loggedIn = value
        }

    override fun onCreate() {
        super.onCreate()

        notificationManager = ForegroundNotificationManager(this, serviceNotifier)
        tunnelStateUpdater = TunnelStateUpdater(this, serviceNotifier)

        setUp()
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        val startResult = super.onStartCommand(intent, flags, startId)
        val action = intent?.action

        if (action == VpnService.SERVICE_INTERFACE || action == KEY_CONNECT_ACTION) {
            if (loggedIn) {
                pendingAction = PendingAction.Connect
            } else {
                pendingAction = null
                openUi()
            }
        } else if (action == KEY_DISCONNECT_ACTION) {
            pendingAction = PendingAction.Disconnect
        }

        return startResult
    }

    override fun onBind(intent: Intent): IBinder {
        bindCount += 1

        return super.onBind(intent) ?: binder
    }

    override fun onRebind(intent: Intent) {
        bindCount += 1

        if (isStopping) {
            restart()
            isStopping = false
        }
    }

    override fun onRevoke() {
        pendingAction = PendingAction.Disconnect
    }

    override fun onUnbind(intent: Intent): Boolean {
        bindCount -= 1

        return true
    }

    override fun onDestroy() {
        tearDown()
        notificationManager.onDestroy()
        super.onDestroy()
    }

    inner class LocalBinder : Binder() {
        val serviceNotifier
            get() = this@MullvadVpnService.serviceNotifier

        fun stop() {
            this@MullvadVpnService.stop()
        }
    }

    private fun setUp() {
        startDaemonJob?.cancel()
        startDaemonJob = startDaemon()
    }

    private fun startDaemon() = GlobalScope.launch(Dispatchers.Default) {
        FileResourceExtractor(API_ROOT_CA_FILE, API_ROOT_CA_PATH)
            .extract(application)

        FileResourceExtractor(RELAYS_FILE, RELAYS_PATH)
            .extract(application)

        val newDaemon = MullvadDaemon(this@MullvadVpnService).apply {
            onSettingsChange.subscribe { settings ->
                loggedIn = settings?.accountToken != null
            }

            onDaemonStopped = {
                connectionProxy?.onDestroy()
                serviceNotifier.notify(null)

                if (!isStopping) {
                    restart()
                }
            }
        }

        val newConnectionProxy = ConnectionProxy(this@MullvadVpnService, newDaemon).apply {
            when (pendingAction) {
                PendingAction.Connect -> connect()
                PendingAction.Disconnect -> disconnect()
                null -> {}
            }

            pendingAction = null
        }

        daemon = newDaemon
        connectionProxy = newConnectionProxy

        serviceNotifier.notify(ServiceInstance(newDaemon, newConnectionProxy, connectivityListener))
    }

    private fun stop() {
        isStopping = true
        stopDaemon()
        stopSelf()
    }

    private fun stopDaemon() {
        startDaemonJob?.cancel()
        daemon?.shutdown()
    }

    private fun tearDown() {
        stopDaemon()
    }

    private fun restart() {
        tearDown()
        setUp()
    }

    private fun openUi() {
        val intent = Intent(this, MainActivity::class.java).apply {
            addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
        }

        startActivity(intent)
    }
}
