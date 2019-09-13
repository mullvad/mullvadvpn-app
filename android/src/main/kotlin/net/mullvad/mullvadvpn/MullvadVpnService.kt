package net.mullvad.mullvadvpn

import java.net.InetAddress

import kotlinx.coroutines.async
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.Deferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope

import android.content.Intent
import android.net.VpnService
import android.os.Binder
import android.os.IBinder

import net.mullvad.mullvadvpn.dataproxy.AppVersionInfoFetcher
import net.mullvad.mullvadvpn.dataproxy.ConnectionProxy
import net.mullvad.mullvadvpn.model.TunConfig

class MullvadVpnService : VpnService() {
    private val binder = LocalBinder()
    private val created = CompletableDeferred<Unit>()

    private var resetComplete: CompletableDeferred<Unit>? = null

    private lateinit var daemon: Deferred<MullvadDaemon>
    private lateinit var connectionProxy: ConnectionProxy
    private lateinit var notificationManager: ForegroundNotificationManager
    private lateinit var versionInfoFetcher: AppVersionInfoFetcher

    override fun onCreate() {
        setUp()
        created.complete(Unit)
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
        created.cancel()
    }

    fun createTun(config: TunConfig): Int {
        val builder = Builder().apply {
            for (address in config.addresses) {
                addAddress(address, 32)
            }

            for (dnsServer in config.dnsServers) {
                addDnsServer(dnsServer)
            }

            for (route in config.routes) {
                addRoute(route.address, route.prefixLength.toInt())
            }

            setMtu(config.mtu)
        }

        val vpnInterface = builder.establish()

        return vpnInterface.detachFd()
    }

    fun bypass(socket: Int): Boolean {
        return protect(socket)
    }

    inner class LocalBinder : Binder() {
        val daemon
            get() = this@MullvadVpnService.daemon
        val connectionProxy
            get() = this@MullvadVpnService.connectionProxy
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
        versionInfoFetcher = AppVersionInfoFetcher(daemon, this)
    }

    private fun startDaemon() = GlobalScope.async(Dispatchers.Default) {
        created.await()
        ApiRootCaFile().extract(application)
        MullvadDaemon(this@MullvadVpnService)
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
        versionInfoFetcher.stop()
    }
}
