package net.mullvad.talpid

import android.os.ParcelFileDescriptor
import androidx.annotation.CallSuper
import co.touchlab.kermit.Logger
import java.net.Inet4Address
import java.net.Inet6Address
import java.net.InetAddress
import kotlin.properties.Delegates.observable
import net.mullvad.mullvadvpn.lib.common.util.prepareVpnSafe
import net.mullvad.mullvadvpn.lib.model.PrepareError
import net.mullvad.talpid.model.CreateTunResult
import net.mullvad.talpid.model.TunConfig
import net.mullvad.talpid.util.TalpidSdkUtils.setMeteredIfSupported

open class TalpidVpnService : LifecycleVpnService() {
    private var activeTunStatus by
        observable<CreateTunResult?>(null) { _, oldTunStatus, _ ->
            val oldTunFd =
                when (oldTunStatus) {
                    is CreateTunResult.Success -> oldTunStatus.tunFd
                    is CreateTunResult.InvalidDnsServers -> oldTunStatus.tunFd
                    else -> null
                }

            if (oldTunFd != null) {
                ParcelFileDescriptor.adoptFd(oldTunFd).close()
            }
        }

    private var currentTunConfig: TunConfig? = null

    // Used by JNI
    val connectivityListener = ConnectivityListener()

    @CallSuper
    override fun onCreate() {
        super.onCreate()
        connectivityListener.register(this)
    }

    @CallSuper
    override fun onDestroy() {
        super.onDestroy()
        connectivityListener.unregister()
    }

    fun openTun(config: TunConfig): CreateTunResult {
        synchronized(this) {
            val tunStatus = activeTunStatus

            if (config == currentTunConfig && tunStatus != null && tunStatus.isOpen) {
                return tunStatus
            } else {
                return openTunImpl(config)
            }
        }
    }

    fun openTunForced(config: TunConfig): CreateTunResult {
        synchronized(this) {
            return openTunImpl(config)
        }
    }

    private fun openTunImpl(config: TunConfig): CreateTunResult {
        val newTunStatus = createTun(config)

        currentTunConfig = config
        activeTunStatus = newTunStatus

        return newTunStatus
    }

    fun closeTun() {
        synchronized(this) { activeTunStatus = null }
    }

    // DROID-1407
    // Function is to be cleaned up and lint suppression to be removed.
    @Suppress("ReturnCount")
    private fun createTun(config: TunConfig): CreateTunResult {
        prepareVpnSafe()
            .mapLeft {
                when (it) {
                    is PrepareError.LegacyLockdown -> CreateTunResult.LegacyLockdown
                    is PrepareError.NotPrepared ->
                        CreateTunResult.NotPrepared(
                            it.prepareIntent.component!!.packageName,
                            it.prepareIntent.component!!.className,
                        )
                    is PrepareError.OtherAlwaysOnApp -> CreateTunResult.AlwaysOnApp(it.appName)
                }
            }
            .onLeft {
                return it
            }

        val invalidDnsServerAddresses = ArrayList<InetAddress>()

        val builder =
            Builder().apply {
                for (address in config.addresses) {
                    addAddress(address, address.prefixLength())
                }

                for (dnsServer in config.dnsServers) {
                    try {
                        addDnsServer(dnsServer)
                    } catch (exception: IllegalArgumentException) {
                        invalidDnsServerAddresses.add(dnsServer)
                    }
                }

                // Avoids creating a tunnel with no DNS servers or if all DNS servers was invalid,
                // since apps then may leak DNS requests.
                // https://issuetracker.google.com/issues/337961996
                if (invalidDnsServerAddresses.size == config.dnsServers.size) {
                    Logger.w(
                        "All DNS servers invalid or non set, using fallback DNS server to " +
                            "minimize leaks, dnsServers.isEmpty(): ${config.dnsServers.isEmpty()}"
                    )
                    addDnsServer(FALLBACK_DUMMY_DNS_SERVER)
                }

                for (route in config.routes) {
                    addRoute(route.address, route.prefixLength.toInt())
                }

                config.excludedPackages.forEach { app -> addDisallowedApplication(app) }
                setMtu(config.mtu)
                setBlocking(false)
                setMeteredIfSupported(false)
            }

        val vpnInterfaceFd =
            try {
                builder.establish()
            } catch (e: IllegalStateException) {
                Logger.e("Failed to establish, a parameter could not be applied", e)
                return CreateTunResult.TunnelDeviceError
            } catch (e: IllegalArgumentException) {
                Logger.e("Failed to establish a parameter was not accepted", e)
                return CreateTunResult.TunnelDeviceError
            }

        if (vpnInterfaceFd == null) {
            Logger.e("VpnInterface returned null")
            return CreateTunResult.TunnelDeviceError
        }

        val tunFd = vpnInterfaceFd.detachFd()

        waitForTunnelUp(tunFd, config.routes.any { route -> route.isIpv6 })

        if (invalidDnsServerAddresses.isNotEmpty()) {
            return CreateTunResult.InvalidDnsServers(invalidDnsServerAddresses, tunFd)
        }

        return CreateTunResult.Success(tunFd)
    }

    fun bypass(socket: Int): Boolean {
        return protect(socket)
    }

    private fun InetAddress.prefixLength(): Int =
        when (this) {
            is Inet4Address -> IPV4_PREFIX_LENGTH
            is Inet6Address -> IPV6_PREFIX_LENGTH
            else -> throw IllegalArgumentException("Invalid IP address (not IPv4 nor IPv6)")
        }

    private external fun waitForTunnelUp(tunFd: Int, isIpv6Enabled: Boolean)

    companion object {
        private const val FALLBACK_DUMMY_DNS_SERVER = "192.0.2.1"

        private const val IPV4_PREFIX_LENGTH = 32
        private const val IPV6_PREFIX_LENGTH = 128
    }
}
