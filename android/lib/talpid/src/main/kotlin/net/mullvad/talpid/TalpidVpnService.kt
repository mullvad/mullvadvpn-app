package net.mullvad.talpid

import android.net.ConnectivityManager
import android.os.ParcelFileDescriptor
import androidx.annotation.CallSuper
import androidx.core.content.getSystemService
import androidx.lifecycle.lifecycleScope
import arrow.core.Either
import arrow.core.Nel
import arrow.core.left
import arrow.core.mapOrAccumulate
import arrow.core.maxBy
import arrow.core.merge
import arrow.core.raise.ExperimentalRaiseAccumulateApi
import arrow.core.raise.accumulate
import arrow.core.raise.either
import arrow.core.right
import co.touchlab.kermit.Logger
import java.net.Inet4Address
import java.net.Inet6Address
import java.net.InetAddress
import kotlin.properties.Delegates.observable
import net.mullvad.mullvadvpn.lib.common.util.establishSafe
import net.mullvad.mullvadvpn.lib.common.util.prepareVpnSafe
import net.mullvad.mullvadvpn.lib.model.PrepareError
import net.mullvad.talpid.model.CreateTunResult
import net.mullvad.talpid.model.CreateTunResult.EstablishError
import net.mullvad.talpid.model.CreateTunResult.InvalidDnsServers
import net.mullvad.talpid.model.CreateTunResult.NotPrepared
import net.mullvad.talpid.model.CreateTunResult.OtherAlwaysOnApp
import net.mullvad.talpid.model.CreateTunResult.OtherLegacyAlwaysOnVpn
import net.mullvad.talpid.model.InetNetwork
import net.mullvad.talpid.model.TunConfig
import net.mullvad.talpid.util.TalpidSdkUtils.setMeteredIfSupported
import net.mullvad.talpid.util.UnderlyingConnectivityStatusResolver

open class TalpidVpnService : LifecycleVpnService() {
    private var activeTunStatus by
        observable<CreateTunResult?>(null) { _, oldTunStatus, _ ->
            val oldTunFd =
                when (oldTunStatus) {
                    is CreateTunResult.Success -> oldTunStatus.tunFd
                    is InvalidDnsServers -> oldTunStatus.tunFd
                    is CreateTunResult.InvalidIpv6Config -> oldTunStatus.tunFd
                    else -> null
                }

            if (oldTunFd != null) {
                ParcelFileDescriptor.adoptFd(oldTunFd).close()
            }
        }

    // Used by JNI
    lateinit var connectivityListener: ConnectivityListener

    @CallSuper
    override fun onCreate() {
        super.onCreate()
        connectivityListener =
            ConnectivityListener(
                getSystemService<ConnectivityManager>()!!,
                UnderlyingConnectivityStatusResolver(::protect),
            )
        connectivityListener.register(lifecycleScope)
    }

    // Used by JNI
    fun openTun(config: TunConfig): CreateTunResult =
        synchronized(this) { createTun(config).merge().also { activeTunStatus = it } }

    // Used by JNI
    @Suppress("Unused")
    fun closeTun(): Unit =
        synchronized(this) {
            connectivityListener.invalidateNetworkStateCache()
            activeTunStatus = null
        }

    // Used by JNI
    @Suppress("Unused") fun bypass(socket: Int): Boolean = protect(socket)

    private fun createTun(
        config: TunConfig
    ): Either<CreateTunResult.Error, CreateTunResult.Success> = either {
        prepareVpnSafe().mapLeft { it.toCreateTunError() }.bind()

        val builder = Builder()
        val configureResult = builder.configureWith(config)

        connectivityListener.invalidateNetworkStateCache()
        val vpnInterfaceFd =
            builder
                .establishSafe()
                .onLeft { Logger.w("Failed to establish tunnel $it") }
                .mapLeft { EstablishError }
                .bind()

        val tunFd = vpnInterfaceFd.detachFd()

        configureResult
            .mapLeft { errors -> errors.maxBy { it.priority }.toCreateTunError(tunFd) }
            .bind()
        CreateTunResult.Success(tunFd)
    }

    @OptIn(ExperimentalRaiseAccumulateApi::class)
    private fun Builder.configureWith(config: TunConfig): Either<Nel<ConfigError>, Unit> = either {
        setMtu(config.mtu)
        setBlocking(false)
        setMeteredIfSupported(false)

        config.excludedPackages.forEach { app -> addDisallowedApplication(app) }

        accumulate {
            accumulating { addAddressesAndRoutesSafe(config).bind() }
            accumulating { addDnsServersSafe(config).bind() }
        }
    }

    // Blocking fallback for when we receive an invalid IPv6 config
    private fun Builder.invalidIpv6Setup() {
        addAddress(BLOCKING_ADDRESS_IPV4, IPV4_PREFIX_LENGTH)
        addAddress(BLOCKING_ADDRESS_IPV6, IPV6_PREFIX_LENGTH)
        addRoute(ROUTE_ALL_IPV4, 0)
        addRoute(ROUTE_ALL_IPV6, 0)
        // IPv6 have a minimum mtu of 1280, the daemon can send a lower mtu if IPv6 is disabled
        // Due to this we need to set a dummy mtu or the establish might fail
        setMtu(BLOCKING_MTU)
    }

    // To avoid leaks a config should either fully contain IPv6 or have no IPv6 configuration. A
    // partial IPv6 configuration, e.g having no address but providing routes, will lead to traffic
    // routed outside the tunnel
    private fun TunConfig.validIpv6Routes(): Boolean =
        hasCompleteIpv6Configuration() || hasNoIpv6Configuration()

    /**
     * Checks for a complete IPv6 configuration, meaning at least one IPv6 address and one IPv6
     * route exist.
     */
    private fun TunConfig.hasCompleteIpv6Configuration() = hasIpv6Route && hasIpv6Address

    /** Checks that no IPv6 configuration (addresses, routes, or DNS) is present. */
    private fun TunConfig.hasNoIpv6Configuration() =
        !hasIpv6Address && !hasIpv6DnsServer && !hasIpv6Route

    private fun PrepareError.toCreateTunError() =
        when (this) {
            is PrepareError.OtherLegacyAlwaysOnVpn -> OtherLegacyAlwaysOnVpn
            is PrepareError.NotPrepared -> NotPrepared
            is PrepareError.OtherAlwaysOnApp -> OtherAlwaysOnApp(appName)
        }

    private fun ConfigError.toCreateTunError(tunFd: Int) =
        when (this) {
            is ConfigError.InvalidIpv6 ->
                CreateTunResult.InvalidIpv6Config(
                    addresses = addresses,
                    routes = routes,
                    dnsServers = dnsServers,
                    tunFd = tunFd,
                )
            is ConfigError.InvalidDns -> InvalidDnsServers(addresses = addresses, tunFd = tunFd)
        }

    private fun Builder.addDnsServersSafe(
        config: TunConfig
    ): Either<ConfigError.InvalidDns, Builder> {
        // We don't care if adding DNS servers fails at this point, since we can still create a
        // tunnel to consume traffic and then notify daemon to later enter blocked state.
        val dnsConfigureResult =
            config.dnsServers.mapOrAccumulate {
                addDnsServerSafe(it).bind()
                Unit
            }

        // Never create a tunnel where all DNS servers are invalid or if none was ever set, since
        // apps then may leak DNS requests.
        // https://issuetracker.google.com/issues/337961996
        val shouldAddFallbackDns =
            dnsConfigureResult.fold(
                { invalidDnsServers -> invalidDnsServers.size == config.dnsServers.size },
                { addedDnsServers -> addedDnsServers.isEmpty() },
            )
        if (shouldAddFallbackDns) {
            Logger.w(
                "All DNS servers invalid or non set, using fallback DNS server to " +
                    "minimize leaks, dnsServers.isEmpty(): ${config.dnsServers.isEmpty()}"
            )
            addDnsServer(FALLBACK_DUMMY_DNS_SERVER)
        }
        return dnsConfigureResult.mapLeft { ConfigError.InvalidDns(addresses = it) }.map { this }
    }

    private fun Builder.addDnsServerSafe(dnsServer: InetAddress): Either<InetAddress, Builder> =
        Either.catch { addDnsServer(dnsServer) }
            .mapLeft {
                when (it) {
                    is IllegalArgumentException -> dnsServer
                    else -> throw it
                }
            }

    private fun Builder.addAddressesAndRoutesSafe(
        config: TunConfig
    ): Either<ConfigError.InvalidIpv6, Builder> =
        if (config.validIpv6Routes()) {
            config.addresses.forEach { addAddress(it, it.prefixLength()) }
            config.routes.forEach { addRoute(it.address, it.prefixLength.toInt()) }
            right()
        } else {
            Logger.e("Bad Ipv6 config provided!")
            Logger.e("IPv6 address: ${config.hasIpv6Address}")
            Logger.e("IPv6 route: ${config.hasIpv6Route}")
            Logger.e("IPv6 DnsServer: ${config.hasIpv6DnsServer}")
            invalidIpv6Setup()
            ConfigError.InvalidIpv6(
                    addresses = config.addresses,
                    routes = config.routes,
                    dnsServers = config.dnsServers,
                )
                .left()
        }

    private fun InetAddress.prefixLength(): Int =
        when (this) {
            is Inet4Address -> IPV4_PREFIX_LENGTH
            is Inet6Address -> IPV6_PREFIX_LENGTH
            else -> throw IllegalArgumentException("Invalid IP address (not IPv4 nor IPv6)")
        }

    companion object {
        const val FALLBACK_DUMMY_DNS_SERVER = "192.0.2.1"
        const val BLOCKING_ADDRESS_IPV4 = "10.0.0.1"
        const val BLOCKING_ADDRESS_IPV6 = "fd00::1"
        const val ROUTE_ALL_IPV4 = "0.0.0.0"
        const val ROUTE_ALL_IPV6 = "::"
        const val BLOCKING_MTU = 1280

        private const val IPV4_PREFIX_LENGTH = 32
        private const val IPV6_PREFIX_LENGTH = 128
    }
}

private sealed interface ConfigError {
    val priority: Int

    data class InvalidDns(val addresses: List<InetAddress>) : ConfigError {
        override val priority: Int = 1
    }

    data class InvalidIpv6(
        val addresses: List<InetAddress>,
        val routes: List<InetNetwork>,
        val dnsServers: List<InetAddress>,
    ) : ConfigError {
        override val priority: Int = 2
    }
}
