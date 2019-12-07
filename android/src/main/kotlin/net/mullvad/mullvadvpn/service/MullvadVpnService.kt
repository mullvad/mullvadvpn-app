package net.mullvad.mullvadvpn.service

import android.content.Intent
import android.os.Binder
import android.os.IBinder
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.Deferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.async
import kotlinx.coroutines.runBlocking
import net.mullvad.mullvadvpn.dataproxy.ConnectionProxy
import net.mullvad.talpid.TalpidVpnService

class MullvadVpnService : TalpidVpnService() {
    private val binder = LocalBinder()

    private var resetComplete: CompletableDeferred<Unit>? = null

    private lateinit var daemon: Deferred<MullvadDaemon>
    private lateinit var connectionProxy: ConnectionProxy
    private lateinit var notificationManager: ForegroundNotificationManager

    override fun onCreate() {
        super.onCreate()
        setUp()
    }

    override fun onBind(intent: Intent): IBinder {
        return super.onBind(intent) ?: binder
    }

    override fun onRebind(intent: Intent) {
        resetComplete?.let { reset ->
            tearDown()
            setUp()
            reset.complete(Unit)
        }
    }

    override fun onUnbind(intent: Intent): Boolean {
        return true
    }

    override fun onDestroy() {
        tearDown()
        daemon.cancel()
        super.onDestroy()
    }

    inner class LocalBinder : Binder() {
        val daemon
            get() = this@MullvadVpnService.daemon
        val connectionProxy
            get() = this@MullvadVpnService.connectionProxy
        val connectivityListener
            get() = this@MullvadVpnService.connectivityListener
        val resetComplete
            get() = this@MullvadVpnService.resetComplete

        fun stop() {
            this@MullvadVpnService.stop()
        }
    }

    private fun setUp() {
        daemon = startDaemon()
        connectionProxy = ConnectionProxy(this, daemon)
        notificationManager = startNotificationManager()
    }

    private fun startDaemon() = GlobalScope.async(Dispatchers.Default) {
        ApiRootCaFile().extract(application)

        MullvadDaemon(this@MullvadVpnService).apply {
            onSettingsChange.subscribe { settings ->
                notificationManager.loggedIn = settings?.accountToken != null
            }
        }
    }

    private fun startNotificationManager(): ForegroundNotificationManager {
        return ForegroundNotificationManager(this, connectionProxy).apply {
            onConnect = { connectionProxy.connect() }
            onDisconnect = { connectionProxy.disconnect() }
        }
    }

    private fun stop() {
        this@MullvadVpnService.resetComplete = CompletableDeferred()

        if (daemon.isCompleted) {
            runBlocking { daemon.await().shutdown() }
        } else {
            daemon.cancel()
        }

        stopSelf()
    }

    private fun tearDown() {
        connectionProxy.onDestroy()
        notificationManager.onDestroy()
    }
}
