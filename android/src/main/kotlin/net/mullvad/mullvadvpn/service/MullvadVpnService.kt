package net.mullvad.mullvadvpn.service

import android.content.Intent
import android.net.VpnService
import android.os.Binder
import android.os.IBinder
import kotlinx.coroutines.Deferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.async
import kotlinx.coroutines.runBlocking
import net.mullvad.mullvadvpn.dataproxy.ConnectionProxy
import net.mullvad.talpid.TalpidVpnService
import net.mullvad.talpid.util.EventNotifier

class MullvadVpnService : TalpidVpnService() {
    private val binder = LocalBinder()
    private val serviceNotifier = EventNotifier<ServiceInstance?>(null)

    private var isStopping = false

    private var connectionProxy: ConnectionProxy? = null

    private lateinit var daemon: Deferred<MullvadDaemon>
    private lateinit var notificationManager: ForegroundNotificationManager

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

    override fun onCreate() {
        super.onCreate()
        notificationManager = ForegroundNotificationManager(this, serviceNotifier)
        setUp()
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
        stop()
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

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        val startResult = super.onStartCommand(intent, flags, startId)
        if (intent?.getAction() == VpnService.SERVICE_INTERFACE) {
            runBlocking { daemon.await().connect() }
        }
        return startResult
    }

    private fun setUp() {
        daemon = startDaemon()
    }

    private fun startDaemon() = GlobalScope.async(Dispatchers.Default) {
        ApiRootCaFile().extract(application)

        val daemon = MullvadDaemon(this@MullvadVpnService).apply {
            onSettingsChange.subscribe { settings ->
                notificationManager.loggedIn = settings?.accountToken != null
            }

            onDaemonStopped = {
                connectionProxy?.onDestroy()
                serviceNotifier.notify(null)

                if (!isStopping) {
                    restart()
                }
            }
        }

        val newConnectionProxy = ConnectionProxy(this@MullvadVpnService, newDaemon)

        connectionProxy = newConnectionProxy

        serviceNotifier.notify(ServiceInstance(daemon, newConnectionProxy, connectivityListener))

        daemon
    }

    private fun stop() {
        isStopping = true
        stopDaemon()
        stopSelf()
    }

    private fun stopDaemon() {
        if (daemon.isCompleted) {
            runBlocking { daemon.await().shutdown() }
        } else {
            daemon.cancel()
        }
    }

    private fun tearDown() {
        stopDaemon()
    }

    private fun restart() {
        tearDown()
        setUp()
    }
}
