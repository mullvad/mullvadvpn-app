package net.mullvad.talpid

import android.net.ConnectivityManager
import android.net.LinkProperties
import android.os.ParcelFileDescriptor
import androidx.annotation.CallSuper
import androidx.core.content.getSystemService
import androidx.lifecycle.lifecycleScope
import arrow.core.Either
import arrow.core.flattenOrAccumulate
import arrow.core.merge
import arrow.core.raise.either
import arrow.core.raise.ensureNotNull
import co.touchlab.kermit.Logger
import java.net.Inet4Address
import java.net.Inet6Address
import java.net.InetAddress
import kotlin.properties.Delegates.observable
import kotlin.time.Duration
import kotlin.time.Duration.Companion.milliseconds
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.withTimeoutOrNull
import net.mullvad.mullvadvpn.lib.common.util.establishSafe
import net.mullvad.mullvadvpn.lib.common.util.prepareVpnSafe
import net.mullvad.mullvadvpn.lib.model.PrepareError
import net.mullvad.talpid.model.CreateTunResult
import net.mullvad.talpid.model.CreateTunResult.EstablishError
import net.mullvad.talpid.model.CreateTunResult.InvalidDnsServers
import net.mullvad.talpid.model.CreateTunResult.NotPrepared
import net.mullvad.talpid.model.CreateTunResult.OtherAlwaysOnApp
import net.mullvad.talpid.model.CreateTunResult.OtherLegacyAlwaysOnVpn
import net.mullvad.talpid.model.CreateTunResult.RoutesTimedOutError
import net.mullvad.talpid.model.InetNetwork
import net.mullvad.talpid.model.TunConfig
import net.mullvad.talpid.util.TalpidSdkUtils.setMeteredIfSupported

open class TalpidVpnService : LifecycleVpnService() {
    private var activeTunStatus by
        observable<CreateTunResult?>(null) { _, oldTunStatus, _ ->
            val oldTunFd =
                when (oldTunStatus) {
                    is CreateTunResult.Success -> oldTunStatus.tunFd
                    is InvalidDnsServers -> oldTunStatus.tunFd
                    else -> null
                }

            if (oldTunFd != null) {
                ParcelFileDescriptor.adoptFd(oldTunFd).close()
            }
        }

    private var currentTunConfig: TunConfig? = null

    // Used by JNI
    lateinit var connectivityListener: ConnectivityListener

    @CallSuper
    override fun onCreate() {
        super.onCreate()
        connectivityListener = ConnectivityListener(getSystemService<ConnectivityManager>()!!)
        connectivityListener.register(lifecycleScope)
    }

    // Used by JNI
    fun openTun(config: TunConfig): CreateTunResult =
        synchronized(this) {
            val tunStatus = activeTunStatus

            if (config == currentTunConfig && tunStatus != null && tunStatus.isOpen) {
                tunStatus
            } else {
                openTunImpl(config)
            }
        }

    // Used by JNI
    fun openTunForced(config: TunConfig): CreateTunResult =
        synchronized(this) { openTunImpl(config) }

    // Used by JNI
    fun closeTun(): Unit = synchronized(this) { activeTunStatus = null }

    // Used by JNI
    fun bypass(socket: Int): Boolean = protect(socket)

    private fun openTunImpl(config: TunConfig): CreateTunResult {
        val newTunStatus = createTun(config).merge()

        currentTunConfig = config
        activeTunStatus = newTunStatus

        return newTunStatus
    }

    private fun createTun(
        config: TunConfig
    ): Either<CreateTunResult.Error, CreateTunResult.Success> = either {
        prepareVpnSafe().mapLeft { it.toCreateTunError() }.bind()

        val builder = Builder()
        builder.setMtu(config.mtu)
        builder.setBlocking(false)
        builder.setMeteredIfSupported(false)

        config.addresses.forEach { builder.addAddress(it, it.prefixLength()) }
        config.routes.forEach { builder.addRoute(it.address, it.prefixLength.toInt()) }
        config.excludedPackages.forEach { app -> builder.addDisallowedApplication(app) }

        // We don't care if this fails at this point, since we can still create a tunnel and
        // then notify daemon to later enter blocked state.
        val dnsConfigureResult =
            config.dnsServers
                .map { builder.addDnsServerE(it) }
                .flattenOrAccumulate()
                .onLeft {
                    // Avoid creating a tunnel with no DNS servers or if all DNS servers was
                    // invalid, since apps then may leak DNS requests.
                    // https://issuetracker.google.com/issues/337961996
                    if (it.size == config.dnsServers.size) {
                        Logger.w(
                            "All DNS servers invalid or non set, using fallback DNS server to " +
                                "minimize leaks, dnsServers.isEmpty(): ${config.dnsServers.isEmpty()}"
                        )
                        builder.addDnsServer(FALLBACK_DUMMY_DNS_SERVER)
                    }
                }
                .map { /* Ignore right */ }

        val vpnInterfaceFd =
            builder
                .establishSafe()
                .onLeft { Logger.w("Failed to establish tunnel $it") }
                .mapLeft { EstablishError }
                .bind()

        // Wait for android OS to respond back to us that the routes are setup so we don't
        // send traffic before the routes are set up. Otherwise we might send traffic
        // through the wrong interface
        runBlocking { waitForRoutesWithTimeout(config) }.bind()

        val tunFd = vpnInterfaceFd.detachFd()

        dnsConfigureResult.mapLeft { InvalidDnsServers(it, tunFd) }.bind()

        CreateTunResult.Success(tunFd)
    }

    private fun PrepareError.toCreateTunError() =
        when (this) {
            is PrepareError.OtherLegacyAlwaysOnVpn -> OtherLegacyAlwaysOnVpn
            is PrepareError.NotPrepared -> NotPrepared
            is PrepareError.OtherAlwaysOnApp -> OtherAlwaysOnApp(appName)
        }

    private fun Builder.addDnsServerE(dnsServer: InetAddress) =
        Either.catch { addDnsServer(dnsServer) }
            .mapLeft {
                when (it) {
                    is IllegalArgumentException -> dnsServer
                    else -> throw it
                }
            }

    private suspend fun waitForRoutesWithTimeout(
        config: TunConfig,
        timeout: Duration = ROUTES_SETUP_TIMEOUT,
    ): Either<RoutesTimedOutError, Unit> = either {
        // Wait for routes to match our expectations
        val result =
            withTimeoutOrNull(timeout = timeout) {
                connectivityListener.currentNetworkState
                    .map { it?.linkProperties }
                    .first { linkProps -> linkProps?.containsAll(config.routes) == true }
            }

        ensureNotNull(result) { RoutesTimedOutError }
    }

    private fun LinkProperties.containsAll(configRoutes: List<InetNetwork>): Boolean {
        // Current routes on the link
        val linkRoutes =
            routes.map {
                InetNetwork(it.destination.address, it.destination.prefixLength.toShort())
            }

        return linkRoutes.containsAll(configRoutes)
    }

    private fun InetAddress.prefixLength(): Int =
        when (this) {
            is Inet4Address -> IPV4_PREFIX_LENGTH
            is Inet6Address -> IPV6_PREFIX_LENGTH
            else -> throw IllegalArgumentException("Invalid IP address (not IPv4 nor IPv6)")
        }

    companion object {
        const val FALLBACK_DUMMY_DNS_SERVER = "192.0.2.1"
        private val ROUTES_SETUP_TIMEOUT = 5000.milliseconds

        private const val IPV4_PREFIX_LENGTH = 32
        private const val IPV6_PREFIX_LENGTH = 128
    }
}
