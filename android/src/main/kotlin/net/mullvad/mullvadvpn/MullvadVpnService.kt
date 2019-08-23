package net.mullvad.mullvadvpn

import java.net.InetAddress

import kotlinx.coroutines.async
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope

import android.app.Activity
import android.app.Notification
import android.app.PendingIntent
import android.content.Context
import android.content.Intent
import android.net.VpnService
import android.os.Binder
import android.os.IBinder
import android.support.v4.app.NotificationCompat

import net.mullvad.mullvadvpn.dataproxy.AppVersionInfoFetcher
import net.mullvad.mullvadvpn.model.TunConfig

val ONGOING_NOTIFICATION_ID: Int = 1

class MullvadVpnService : VpnService() {
    private val created = CompletableDeferred<Unit>()
    private val binder = LocalBinder()

    private lateinit var versionInfoFetcher: AppVersionInfoFetcher

    val daemon = startDaemon()

    override fun onCreate() {
        versionInfoFetcher = AppVersionInfoFetcher(daemon, this)
        startForeground(ONGOING_NOTIFICATION_ID, buildNotification())
        created.complete(Unit)
    }

    override fun onBind(intent: Intent): IBinder {
        return super.onBind(intent) ?: binder
    }

    override fun onDestroy() {
        versionInfoFetcher.stop()
        daemon.cancel()
        created.cancel()
        stopForeground(ONGOING_NOTIFICATION_ID)
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

        fun stop() {
            if (daemon.isCompleted) {
                runBlocking { daemon.await().shutdown() }
            } else {
                daemon.cancel()
            }

            stopSelf()
        }
    }

    private fun buildNotification(): Notification {
        val intent = Intent(this, MainActivity::class.java)
            .setFlags(Intent.FLAG_ACTIVITY_CLEAR_TOP or Intent.FLAG_ACTIVITY_SINGLE_TOP)
            .setAction(Intent.ACTION_MAIN)

        val pendingIntent =
            PendingIntent.getActivity(this, 1, intent, PendingIntent.FLAG_UPDATE_CURRENT)

        return NotificationCompat.Builder(this)
            .setSmallIcon(R.drawable.notification)
            .setContentIntent(pendingIntent)
            .build()
    }

    private fun startDaemon() = GlobalScope.async(Dispatchers.Default) {
        created.await()
        ApiRootCaFile().extract(application)
        MullvadDaemon(this@MullvadVpnService)
    }
}
