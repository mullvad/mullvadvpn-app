package net.mullvad.talpid

import android.net.ConnectivityManager
import android.net.VpnService
import android.os.ParcelFileDescriptor
import androidx.annotation.CallSuper
import androidx.core.content.getSystemService
import androidx.lifecycle.lifecycleScope
import arrow.core.Either
import arrow.core.mapOrAccumulate
import arrow.core.merge
import arrow.core.raise.either
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
    fun closeTun(): Unit =
        synchronized(this) {
            connectivityListener.invalidateNetworkStateCache()
            activeTunStatus = null
        }

    // Used by JNI
    fun bypass(socket: Int): Boolean = protect(socket)

    private fun createTun(
        config: TunConfig
    ): Either<CreateTunResult.Error, CreateTunResult.Success> = either {
        prepareVpnSafe().mapLeft { it.toCreateTunError() }.bind()

        Logger.d("TunConfig: $config")

        val builder = Builder()
        builder.setMtu(config.mtu)
        builder.setBlocking(false)
        builder.setMeteredIfSupported(false)

        config.addresses.forEach { builder.addAddress(it, it.prefixLength()) }
        config.routes.forEach { builder.addRoute(it.address, it.prefixLength.toInt()) }
        config.excludedPackages.forEach { app -> builder.addDisallowedApplication(app) }

        // We don't care if adding DNS servers fails at this point, since we can still create a
        // tunnel to consume traffic and then notify daemon to later enter blocked state.
        val dnsConfigureResult =
            config.dnsServers.mapOrAccumulate {
                builder.addDnsServerSafe(it).bind()
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
            builder.addDnsServer(FALLBACK_DUMMY_DNS_SERVER)
        }

        connectivityListener.invalidateNetworkStateCache()
        val vpnInterfaceFd =
            builder
                .establishSafe()
                .onLeft { Logger.w("Failed to establish tunnel $it") }
                .mapLeft { EstablishError }
                .bind()

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

    private fun Builder.addDnsServerSafe(
        dnsServer: InetAddress
    ): Either<InetAddress, VpnService.Builder> =
        Either.catch { addDnsServer(dnsServer) }
            .mapLeft {
                when (it) {
                    is IllegalArgumentException -> dnsServer
                    else -> throw it
                }
            }

    private fun InetAddress.prefixLength(): Int =
        when (this) {
            is Inet4Address -> IPV4_PREFIX_LENGTH
            is Inet6Address -> IPV6_PREFIX_LENGTH
            else -> throw IllegalArgumentException("Invalid IP address (not IPv4 nor IPv6)")
        }

    companion object {
        const val FALLBACK_DUMMY_DNS_SERVER = "192.0.2.1"

        private const val IPV4_PREFIX_LENGTH = 32
        private const val IPV6_PREFIX_LENGTH = 128
    }
}
