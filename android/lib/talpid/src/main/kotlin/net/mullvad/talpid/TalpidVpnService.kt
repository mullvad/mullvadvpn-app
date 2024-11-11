package net.mullvad.talpid

import android.net.ConnectivityManager
import android.net.LinkProperties
import android.os.ParcelFileDescriptor
import android.system.Os.socket
import androidx.annotation.CallSuper
import androidx.core.content.getSystemService
import androidx.lifecycle.lifecycleScope
import co.touchlab.kermit.Logger
import java.net.Inet4Address
import java.net.Inet6Address
import java.net.InetAddress
import kotlin.properties.Delegates.observable
import kotlin.time.Duration
import kotlin.time.Duration.Companion.milliseconds
import kotlin.time.measureTimedValue
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.filterIsInstance
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.withTimeoutOrNull
import net.mullvad.talpid.model.CreateTunResult
import net.mullvad.talpid.model.TunConfig
import net.mullvad.talpid.util.NetworkEvent
import net.mullvad.talpid.util.TalpidSdkUtils.setMeteredIfSupported
import net.mullvad.talpid.util.defaultCallbackFlow

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

    private lateinit var defaultNetworkLinkProperties:
        StateFlow<NetworkEvent.OnLinkPropertiesChanged?>

    @CallSuper
    override fun onCreate() {
        super.onCreate()
        connectivityListener.register(this)

        val connectivityManager = getSystemService<ConnectivityManager>()!!

        defaultNetworkLinkProperties =
            connectivityManager
                .defaultCallbackFlow()
                .filterIsInstance<NetworkEvent.OnLinkPropertiesChanged>()
                .stateIn(lifecycleScope, SharingStarted.Eagerly, null)
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
        if (prepare(this) != null) {
            // VPN permission wasn't granted
            return CreateTunResult.PermissionDenied
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
                    } catch (_: IllegalArgumentException) {
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

        Logger.d("Vpn Interface Established")

        if (vpnInterfaceFd == null) {
            Logger.e("VpnInterface returned null")
            return CreateTunResult.TunnelDeviceError
        }

        // Wait for android OS to respond back to us that the routes are setup so we don't send
        // traffic before the routes are set up. Otherwise we might send traffic through the wrong
        // interface
        runBlocking { waitForRoutesWithTimeout(config) }

        val tunFd = vpnInterfaceFd.detachFd()
        if (invalidDnsServerAddresses.isNotEmpty()) {
            return CreateTunResult.InvalidDnsServers(invalidDnsServerAddresses, tunFd)
        }

        return CreateTunResult.Success(tunFd)
    }

    fun bypass(socket: Int): Boolean {
        return protect(socket)
    }

    @OptIn(ExperimentalCoroutinesApi::class)
    private suspend fun waitForRoutesWithTimeout(
        config: TunConfig,
        timeout: Duration = ROUTES_SETUP_TIMEOUT,
    ) {
        val linkProperties =
            withTimeoutOrNull(timeout = timeout) {
                measureTimedValue {
                        defaultNetworkLinkProperties.filterNotNull().first {
                            it.linkProperties.matches(config)
                        }
                    }
                    .also { Logger.d("LinkProperties matching tunnel, took ${it.duration}") }
                    .value
            }
        if (linkProperties == null) {
            Logger.w("Waiting for LinkProperties timed out")
        }
    }

    // return true if LinkProperties matches the TunConfig
    private fun LinkProperties.matches(tunConfig: TunConfig): Boolean =
        linkAddresses.all { it.address in tunConfig.addresses }

    private fun InetAddress.prefixLength(): Int =
        when (this) {
            is Inet4Address -> IPV4_PREFIX_LENGTH
            is Inet6Address -> IPV6_PREFIX_LENGTH
            else -> throw IllegalArgumentException("Invalid IP address (not IPv4 nor IPv6)")
        }

    companion object {
        private const val FALLBACK_DUMMY_DNS_SERVER = "192.0.2.1"

        private const val IPV4_PREFIX_LENGTH = 32
        private const val IPV6_PREFIX_LENGTH = 128

        private val ROUTES_SETUP_TIMEOUT: Duration = 400.milliseconds
    }
}
