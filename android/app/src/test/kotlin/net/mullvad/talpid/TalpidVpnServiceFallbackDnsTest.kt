package net.mullvad.talpid

import android.net.VpnService
import android.os.ParcelFileDescriptor
import arrow.core.right
import io.mockk.MockKAnnotations
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import io.mockk.mockkConstructor
import io.mockk.mockkStatic
import io.mockk.spyk
import java.net.InetAddress
import net.mullvad.mullvadvpn.lib.common.test.assertLists
import net.mullvad.mullvadvpn.lib.common.util.prepareVpnSafe
import net.mullvad.mullvadvpn.lib.model.Prepared
import net.mullvad.talpid.model.CreateTunResult
import net.mullvad.talpid.model.InetNetwork
import net.mullvad.talpid.model.TunConfig
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.assertInstanceOf

class TalpidVpnServiceFallbackDnsTest {
    lateinit var talpidVpnService: TalpidVpnService
    var builderMockk = mockk<VpnService.Builder>()

    @BeforeEach
    fun setup() {
        MockKAnnotations.init(this)
        mockkStatic(VPN_SERVICE_EXTENSION)

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
        every {
            anyConstructed<VpnService.Builder>()
                .addDnsServer(TalpidVpnService.FALLBACK_DUMMY_DNS_SERVER)
        } returns builderMockk
        val parcelFileDescriptor: ParcelFileDescriptor = mockk()
        every { anyConstructed<VpnService.Builder>().establish() } returns parcelFileDescriptor
        every { parcelFileDescriptor.detachFd() } returns 1
    }

    @Test
    fun `opening tun with no DnsServers should add fallback DNS server`() {
        val tunConfig = baseTunConfig.copy(dnsServers = arrayListOf())

        val result = talpidVpnService.openTun(tunConfig)

        assertInstanceOf<CreateTunResult.Success>(result)

        // Fallback DNS server should be added if no DNS servers are provided
        coVerify(exactly = 1) {
            anyConstructed<VpnService.Builder>()
                .addDnsServer(TalpidVpnService.FALLBACK_DUMMY_DNS_SERVER)
        }
    }

    @Test
    fun `opening tun with all bad DnsServers should return InvalidDnsServers and add fallback`() {
        val badDns1 = InetAddress.getByName("0.0.0.0")
        val badDns2 = InetAddress.getByName("255.255.255.255")
        every { anyConstructed<VpnService.Builder>().addDnsServer(badDns1) } throws
            IllegalArgumentException()
        every { anyConstructed<VpnService.Builder>().addDnsServer(badDns2) } throws
            IllegalArgumentException()

        val tunConfig = baseTunConfig.copy(dnsServers = arrayListOf(badDns1, badDns2))
        val result = talpidVpnService.openTun(tunConfig)

        assertInstanceOf<CreateTunResult.InvalidDnsServers>(result)
        assertLists(tunConfig.dnsServers, result.addresses)
        // Fallback DNS server should be added if no valid DNS servers are provided
        coVerify(exactly = 1) {
            anyConstructed<VpnService.Builder>()
                .addDnsServer(TalpidVpnService.FALLBACK_DUMMY_DNS_SERVER)
        }
    }

    @Test
    fun `opening tun with 1 good and 1 bad DnsServers should return InvalidDnsServers`() {
        val goodDnsServer = InetAddress.getByName("1.1.1.1")
        val badDns = InetAddress.getByName("255.255.255.255")
        every { anyConstructed<VpnService.Builder>().addDnsServer(goodDnsServer) } returns
            builderMockk
        every { anyConstructed<VpnService.Builder>().addDnsServer(badDns) } throws
            IllegalArgumentException()

        val tunConfig = baseTunConfig.copy(dnsServers = arrayListOf(goodDnsServer, badDns))
        val result = talpidVpnService.openTun(tunConfig)

        assertInstanceOf<CreateTunResult.InvalidDnsServers>(result)
        assertLists(arrayListOf(badDns), result.addresses)

        // Fallback DNS server should not be added since we have 1 good DNS server
        coVerify(exactly = 0) {
            anyConstructed<VpnService.Builder>()
                .addDnsServer(TalpidVpnService.FALLBACK_DUMMY_DNS_SERVER)
        }
    }

    @Test
    fun `providing good dns servers should not add the fallback dns and return success`() {
        val goodDnsServer = InetAddress.getByName("1.1.1.1")
        every { anyConstructed<VpnService.Builder>().addDnsServer(goodDnsServer) } returns
            builderMockk

        val tunConfig = baseTunConfig.copy(dnsServers = arrayListOf(goodDnsServer))
        val result = talpidVpnService.openTun(tunConfig)

        assertInstanceOf<CreateTunResult.Success>(result)

        // Fallback DNS server should not be added since we have good DNS servers.
        coVerify(exactly = 0) {
            anyConstructed<VpnService.Builder>()
                .addDnsServer(TalpidVpnService.FALLBACK_DUMMY_DNS_SERVER)
        }
    }

    companion object {
        private const val VPN_SERVICE_EXTENSION =
            "net.mullvad.mullvadvpn.lib.common.util.VpnServiceUtilsKt"

        val baseTunConfig =
            TunConfig(
                addresses = arrayListOf(InetAddress.getByName("45.83.223.209")),
                dnsServers = arrayListOf(),
                routes =
                    arrayListOf(
                        InetNetwork(InetAddress.getByName("0.0.0.0"), 0),
                        InetNetwork(InetAddress.getByName("::"), 0),
                    ),
                mtu = 1280,
                excludedPackages = arrayListOf(),
            )
    }
}
