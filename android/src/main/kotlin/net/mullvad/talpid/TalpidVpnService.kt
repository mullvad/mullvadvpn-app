package net.mullvad.talpid

import android.net.VpnService
import android.os.Build
import android.os.ParcelFileDescriptor
import java.net.Inet4Address
import java.net.Inet6Address
import java.net.InetAddress
import kotlin.properties.Delegates.observable
import net.mullvad.talpid.tun_provider.TunConfig

open class TalpidVpnService : VpnService() {
    private var activeTunDevice by observable<Int?>(null) { _, oldTunDevice, _ ->
        oldTunDevice?.let { oldTunFd ->
            ParcelFileDescriptor.adoptFd(oldTunFd).close()
        }
    }

    private var currentTunConfig = defaultTunConfig()
    private var tunIsStale = false

    protected var disallowedApps: List<String>? = null

    val connectivityListener = ConnectivityListener()

    override fun onCreate() {
        connectivityListener.register(this)
    }

    override fun onDestroy() {
        connectivityListener.unregister(this)
    }

    fun getTun(config: TunConfig): Int {
        synchronized(this) {
            val tunDevice = activeTunDevice

            if (config == currentTunConfig && tunDevice != null && !tunIsStale) {
                return tunDevice
            } else {
                val newTunDevice = createTun(config)

                currentTunConfig = config
                activeTunDevice = newTunDevice
                tunIsStale = false

                return newTunDevice
            }
        }
    }

    fun createTun() {
        synchronized(this) {
            activeTunDevice = createTun(currentTunConfig)
        }
    }

    fun createTunIfClosed() {
        synchronized(this) {
            if (activeTunDevice == null) {
                activeTunDevice = createTun(currentTunConfig)
            }
        }
    }

    fun recreateTunIfOpen(config: TunConfig) {
        synchronized(this) {
            if (activeTunDevice != null) {
                currentTunConfig = config
                activeTunDevice = createTun(config)
            }
        }
    }

    fun closeTun() {
        synchronized(this) {
            activeTunDevice = null
        }
    }

    fun markTunAsStale() {
        synchronized(this) {
            tunIsStale = true
        }
    }

    private fun createTun(config: TunConfig): Int {
        if (VpnService.prepare(this) != null) {
            // VPN permission wasn't granted
            return -1
        }

        val builder = Builder().apply {
            for (address in config.addresses) {
                addAddress(address, prefixForAddress(address))
            }

            for (dnsServer in config.dnsServers) {
                addDnsServer(dnsServer)
            }

            for (route in config.routes) {
                addRoute(route.address, route.prefixLength.toInt())
            }

            disallowedApps?.let { apps ->
                for (app in apps) {
                    addDisallowedApplication(app)
                }
            }

            if (Build.VERSION.SDK_INT >= 29) {
                setMetered(false)
            }

            setMtu(config.mtu)
            setBlocking(false)
        }

        val vpnInterface = builder.establish()
        val tunFd = vpnInterface?.detachFd()

        if (tunFd != null) {
            waitForTunnelUp(tunFd, config.routes.any { route -> route.isIpv6 })
            return tunFd
        } else {
            return 0
        }
    }

    fun bypass(socket: Int): Boolean {
        return protect(socket)
    }

    private fun prefixForAddress(address: InetAddress): Int {
        when (address) {
            is Inet4Address -> return 32
            is Inet6Address -> return 128
            else -> throw RuntimeException("Invalid IP address (not IPv4 nor IPv6)")
        }
    }

    private external fun defaultTunConfig(): TunConfig
    private external fun waitForTunnelUp(tunFd: Int, isIpv6Enabled: Boolean)
}
