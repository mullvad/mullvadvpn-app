package net.mullvad.talpid

import android.net.VpnService
import android.os.ParcelFileDescriptor
import java.net.Inet4Address
import java.net.Inet6Address
import java.net.InetAddress
import kotlin.properties.Delegates.observable
import net.mullvad.mullvadvpn.util.SdkUtils.setMeteredIfSupported
import net.mullvad.talpid.tun_provider.TunConfig

open class TalpidVpnService : VpnService() {
    private var activeTunStatus by observable<CreateTunResult?>(null) { _, oldTunStatus, _ ->
        val oldTunFd = when (oldTunStatus) {
            is CreateTunResult.Success -> oldTunStatus.tunFd
            is CreateTunResult.InvalidDnsServers -> oldTunStatus.tunFd
            else -> null
        }

        if (oldTunFd != null) {
            ParcelFileDescriptor.adoptFd(oldTunFd).close()
        }
    }

    private val tunIsOpen
        get() = activeTunStatus?.isOpen ?: false

    private var currentTunConfig = defaultTunConfig()
    private var tunIsStale = false

    protected var disallowedApps: List<String>? = null

    val connectivityListener = ConnectivityListener()

    override fun onCreate() {
        connectivityListener.register(this)
    }

    override fun onDestroy() {
        connectivityListener.unregister()
    }

    fun getTun(config: TunConfig): CreateTunResult {
        synchronized(this) {
            val tunStatus = activeTunStatus

            if (config == currentTunConfig && tunIsOpen && !tunIsStale) {
                return tunStatus!!
            } else {
                val newTunStatus = createTun(config)

                currentTunConfig = config
                activeTunStatus = newTunStatus
                tunIsStale = false

                return newTunStatus
            }
        }
    }

    fun createTun() {
        synchronized(this) {
            activeTunStatus = createTun(currentTunConfig)
        }
    }

    fun recreateTunIfOpen(config: TunConfig) {
        synchronized(this) {
            if (tunIsOpen) {
                currentTunConfig = config
                activeTunStatus = createTun(config)
            }
        }
    }

    fun closeTun() {
        synchronized(this) {
            activeTunStatus = null
        }
    }

    fun markTunAsStale() {
        synchronized(this) {
            tunIsStale = true
        }
    }

    private fun createTun(config: TunConfig): CreateTunResult {
        if (VpnService.prepare(this) != null) {
            // VPN permission wasn't granted
            return CreateTunResult.PermissionDenied
        }

        var invalidDnsServerAddresses = ArrayList<InetAddress>()

        val builder = Builder().apply {
            for (address in config.addresses) {
                addAddress(address, prefixForAddress(address))
            }

            for (dnsServer in config.dnsServers) {
                try {
                    addDnsServer(dnsServer)
                } catch (exception: IllegalArgumentException) {
                    invalidDnsServerAddresses.add(dnsServer)
                }
            }

            for (route in config.routes) {
                addRoute(route.address, route.prefixLength.toInt())
            }

            disallowedApps?.let { apps ->
                for (app in apps) {
                    addDisallowedApplication(app)
                }
            }
            setMtu(config.mtu)
            setBlocking(false)
            setMeteredIfSupported(false)
        }

        val vpnInterface = builder.establish()
        val tunFd = vpnInterface?.detachFd()

        if (tunFd == null) {
            return CreateTunResult.TunnelDeviceError
        }

        waitForTunnelUp(tunFd, config.routes.any { route -> route.isIpv6 })

        if (!invalidDnsServerAddresses.isEmpty()) {
            return CreateTunResult.InvalidDnsServers(invalidDnsServerAddresses, tunFd)
        }

        return CreateTunResult.Success(tunFd)
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
