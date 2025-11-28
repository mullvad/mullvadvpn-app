package net.mullvad.talpid

import android.net.VpnService
import android.os.Looper
import android.os.ParcelFileDescriptor
import arrow.core.right
import io.mockk.MockKAnnotations
import io.mockk.every
import io.mockk.mockk
import io.mockk.mockkConstructor
import io.mockk.mockkStatic
import io.mockk.spyk
import java.net.Inet6Address
import java.net.InetAddress
import net.mullvad.mullvadvpn.lib.common.util.prepareVpnSafe
import net.mullvad.mullvadvpn.lib.model.Prepared
import net.mullvad.talpid.model.CreateTunResult
import net.mullvad.talpid.model.InetNetwork
import net.mullvad.talpid.model.TunConfig
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.assertInstanceOf

class TalpidVpnServiceInvalidIpv6ConfigTest {
    lateinit var talpidVpnService: TalpidVpnService
    var builderMockk = mockk<VpnService.Builder>()

    @BeforeEach
    fun setup() {
        MockKAnnotations.init(this)
        mockkStatic(VPN_SERVICE_EXTENSION)
        mockkStatic(Looper::class)
        every { Looper.getMainLooper() } returns mockk()
        talpidVpnService = spyk<TalpidVpnService>(recordPrivateCalls = true)
        every { talpidVpnService.prepareVpnSafe() } returns Prepared.right()
        builderMockk = mockk<VpnService.Builder>()

        every { talpidVpnService getProperty "connectivityListener" } returns
            mockk<ConnectivityListener>(relaxed = true)

        mockkConstructor(VpnService.Builder::class)
        every { anyConstructed<VpnService.Builder>().setMtu(any()) } returns builderMockk
        every { anyConstructed<VpnService.Builder>().setBlocking(any()) } returns builderMockk
        every { anyConstructed<VpnService.Builder>().addAddress(any<InetAddress>(), any()) } returns
            builderMockk
        every { anyConstructed<VpnService.Builder>().addRoute(any<InetAddress>(), any()) } returns
            builderMockk
        every { anyConstructed<VpnService.Builder>().addDnsServer(any<InetAddress>()) } returns
            builderMockk
        every {
            anyConstructed<VpnService.Builder>()
                .addDnsServer(TalpidVpnService.FALLBACK_DUMMY_DNS_SERVER)
        } returns builderMockk
        val parcelFileDescriptor: ParcelFileDescriptor = mockk()
        every { anyConstructed<VpnService.Builder>().establish() } returns parcelFileDescriptor
        every { parcelFileDescriptor.detachFd() } returns 1
    }

    @Test
    fun `config with no IPv6 addresses should return Success`() {
        val tunConfig = baseTunConfig

        val result = talpidVpnService.openTun(tunConfig)

        assertInstanceOf<CreateTunResult.Success>(result)
    }

    @Test
    fun `config with complete IPv6 config should return Success`() {
        val tunConfig =
            baseTunConfig.copy(
                addresses =
                    (baseTunConfig.addresses + listOf(Inet6Address.getByName("::")))
                        as ArrayList<InetAddress>,
                routes =
                    (baseTunConfig.routes + listOf(InetNetwork(Inet6Address.getByName("::"), 0)))
                        as ArrayList<InetNetwork>,
            )

        val result = talpidVpnService.openTun(tunConfig)

        assertInstanceOf<CreateTunResult.Success>(result)
    }

    @Test
    fun `config with only a IPv6 route should return InvalidIpv6Config`() {
        val tunConfig =
            baseTunConfig.copy(
                routes =
                    (baseTunConfig.routes + listOf(InetNetwork(Inet6Address.getByName("::"), 0)))
                        as ArrayList<InetNetwork>
            )

        val result = talpidVpnService.openTun(tunConfig)

        assertInstanceOf<CreateTunResult.InvalidIpv6Config>(result)
    }

    @Test
    fun `config with only a IPv6 address should return InvalidIpv6Config`() {
        val tunConfig =
            baseTunConfig.copy(
                addresses =
                    (baseTunConfig.addresses + listOf(Inet6Address.getByName("::")))
                        as ArrayList<InetAddress>
            )

        val result = talpidVpnService.openTun(tunConfig)

        assertInstanceOf<CreateTunResult.InvalidIpv6Config>(result)
    }

    @Test
    fun `config with only a IPv6 dns server should return InvalidIpv6Config`() {
        val tunConfig =
            baseTunConfig.copy(
                dnsServers = mutableListOf(Inet6Address.getByName("::")) as ArrayList<InetAddress>
            )

        val result = talpidVpnService.openTun(tunConfig)

        assertInstanceOf<CreateTunResult.InvalidIpv6Config>(result)
    }

    companion object {
        private const val VPN_SERVICE_EXTENSION =
            "net.mullvad.mullvadvpn.lib.common.util.VpnServiceUtilsKt"

        val baseTunConfig =
            TunConfig(
                addresses = arrayListOf(InetAddress.getByName("45.83.223.209")),
                dnsServers = arrayListOf(InetAddress.getByName("1.1.1.1")),
                routes = arrayListOf(InetNetwork(InetAddress.getByName("0.0.0.0"), 0)),
                mtu = 1280,
                excludedPackages = arrayListOf(),
            )
    }
}
