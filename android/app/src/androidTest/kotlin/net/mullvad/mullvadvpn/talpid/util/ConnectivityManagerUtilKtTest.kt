package net.mullvad.mullvadvpn.talpid.util

import android.net.ConnectivityManager
import android.net.LinkAddress
import android.net.LinkProperties
import android.net.Network
import android.net.NetworkCapabilities
import app.cash.turbine.test
import io.mockk.every
import io.mockk.mockk
import io.mockk.mockkObject
import io.mockk.mockkStatic
import io.mockk.verify
import java.net.DatagramSocket
import java.net.Inet4Address
import java.net.Inet6Address
import kotlin.test.assertEquals
import kotlin.time.Duration.Companion.milliseconds
import kotlin.time.Duration.Companion.seconds
import kotlinx.coroutines.channels.awaitClose
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.callbackFlow
import kotlinx.coroutines.test.runTest
import net.mullvad.talpid.model.Connectivity
import net.mullvad.talpid.util.IpUtils
import net.mullvad.talpid.util.NetworkEvent
import net.mullvad.talpid.util.defaultNetworkEvents
import net.mullvad.talpid.util.hasInternetConnectivity
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test

class ConnectivityManagerUtilKtTest {
    private val connectivityManager = mockk<ConnectivityManager>()

    @BeforeEach
    fun setup() {
        mockkStatic(CONNECTIVITY_MANAGER_UTIL_CLASS)
        mockkObject(IpUtils)
    }

    /** User being online, the listener should emit once with `true` */
    @Test
    fun userIsOnline() = runTest {
        val network = mockk<Network>()
        val linkProperties = mockk<LinkProperties>()
        val ipv4Address: Inet4Address = mockk()
        val ipv6Address: Inet6Address = mockk()
        val linkIpv4Address: LinkAddress = mockk()
        val linkIpv6Address: LinkAddress = mockk()
        every { linkIpv4Address.address } returns ipv4Address
        every { linkIpv6Address.address } returns ipv6Address
        every { linkProperties.linkAddresses } returns
            mutableListOf(linkIpv4Address, linkIpv6Address)
        val mockProtect = mockk<(socket: DatagramSocket) -> Boolean>(relaxed = true)
        every { connectivityManager.defaultNetworkEvents() } returns
            callbackFlow {
                delay(100.milliseconds) // Simulate connectivity listener being a bit slow
                send(NetworkEvent.Available(network))
                delay(100.milliseconds) // Simulate connectivity listener being a bit slow
                send(NetworkEvent.LinkPropertiesChanged(network, linkProperties))
                awaitClose {}
            }

        connectivityManager.hasInternetConnectivity(mockProtect).test {
            // Since initial state and listener both return `true` within debounce we only see one
            // event
            assertEquals(Connectivity.Status(true, true), awaitItem())
            expectNoEvents()
        }
    }

    /** User being offline, the listener should emit once with `false` */
    @Test
    fun userIsOffline() = runTest {
        val mockProtect = mockk<(socket: DatagramSocket) -> Boolean>(relaxed = true)
        every { connectivityManager.defaultNetworkEvents() } returns callbackFlow { awaitClose {} }

        connectivityManager.hasInternetConnectivity(mockProtect).test {
            // Initially offline and no network events, so we should get a single `false` event
            assertEquals(Connectivity.Status(false, false), awaitItem())
            expectNoEvents()
        }
    }

    /** User starting offline and then turning on a online after a while */
    @Test
    fun initiallyOfflineThenBecomingOnline() = runTest {
        val network = mockk<Network>()
        val linkProperties = mockk<LinkProperties>()
        val ipv4Address: Inet4Address = mockk()
        val ipv6Address: Inet6Address = mockk()
        val linkIpv4Address: LinkAddress = mockk()
        val linkIpv6Address: LinkAddress = mockk()
        every { linkIpv4Address.address } returns ipv4Address
        every { linkIpv6Address.address } returns ipv6Address
        every { linkProperties.linkAddresses } returns
            mutableListOf(linkIpv4Address, linkIpv6Address)
        val mockProtect = mockk<(socket: DatagramSocket) -> Boolean>(relaxed = true)
        every { connectivityManager.defaultNetworkEvents() } returns
            callbackFlow {
                // Simulate offline for a little while
                delay(5.seconds)
                // Then become online
                send(NetworkEvent.Available(mockk()))
                send(NetworkEvent.LinkPropertiesChanged(network, linkProperties))
                awaitClose {}
            }

        connectivityManager.hasInternetConnectivity(protect = mockProtect).test {
            assertEquals(Connectivity.Status(false, false), awaitItem())
            assertEquals(Connectivity.Status(true, true), awaitItem())
            expectNoEvents()
        }
    }

    /** User starting online and then becoming offline after 5 seconds */
    @Test
    fun initiallyOnlineAndThenTurningBecomingOffline() = runTest {
        val network = mockk<Network>()
        val linkProperties = mockk<LinkProperties>()
        val ipv4Address: Inet4Address = mockk()
        val ipv6Address: Inet6Address = mockk()
        val linkIpv4Address: LinkAddress = mockk()
        val linkIpv6Address: LinkAddress = mockk()
        every { linkIpv4Address.address } returns ipv4Address
        every { linkIpv6Address.address } returns ipv6Address
        every { linkProperties.linkAddresses } returns
            mutableListOf(linkIpv4Address, linkIpv6Address)
        val mockProtect = mockk<(socket: DatagramSocket) -> Boolean>(relaxed = true)
        every { connectivityManager.defaultNetworkEvents() } returns
            callbackFlow {
                // Starting as online
                send(NetworkEvent.Available(network))
                send(NetworkEvent.LinkPropertiesChanged(network, linkProperties))
                delay(5.seconds)
                // Then becoming offline
                send(NetworkEvent.Lost(network))
                awaitClose {}
            }

        connectivityManager.hasInternetConnectivity(mockProtect).test {
            assertEquals(Connectivity.Status(true, true), awaitItem())
            assertEquals(Connectivity.Status(false, false), awaitItem())
            expectNoEvents()
        }
    }

    /** User roaming from cellular to WiFi. This behavior has been recorded on a Pixel 8 */
    @Test
    fun roamingFromCellularToWifi() = runTest {
        val wifiNetwork = mockk<Network>()
        val wifiNetworkLinkProperties = mockk<LinkProperties>()
        every { wifiNetworkLinkProperties.linkAddresses } returns
            listOf(mockk<LinkAddress> { every { address } returns mockk<Inet4Address>() })
        val cellularNetwork = mockk<Network>()
        val cellularNetworkLinkProperties = mockk<LinkProperties>()
        every { cellularNetworkLinkProperties.linkAddresses } returns
            listOf(mockk<LinkAddress> { every { address } returns mockk<Inet4Address>() })
        val mockProtect = mockk<(socket: DatagramSocket) -> Boolean>(relaxed = true)

        every { connectivityManager.defaultNetworkEvents() } returns
            callbackFlow {
                send(NetworkEvent.Available(cellularNetwork))
                send(
                    NetworkEvent.LinkPropertiesChanged(
                        cellularNetwork,
                        cellularNetworkLinkProperties,
                    )
                )
                delay(5.seconds)
                // Turning on WiFi, we'll have duplicate networks until phone decides to turn of
                // cellular
                send(NetworkEvent.Available(wifiNetwork))
                send(NetworkEvent.LinkPropertiesChanged(wifiNetwork, wifiNetworkLinkProperties))
                delay(30.seconds)
                // Phone turning off cellular network
                send(NetworkEvent.Lost(cellularNetwork))
                awaitClose {}
            }

        connectivityManager.hasInternetConnectivity(mockProtect).test {
            // We should always only see us being online
            assertEquals(Connectivity.Status(ipv4 = true, ipv6 = false), awaitItem())
            expectNoEvents()
        }
    }

    /** User roaming from WiFi to Cellular. This behavior has been recorded on a Pixel 8 */
    @Test
    fun roamingFromWifiToCellular() = runTest {
        val wifiNetwork = mockk<Network>()
        val wifiNetworkLinkProperties = mockk<LinkProperties>()
        every { wifiNetworkLinkProperties.linkAddresses } returns
            listOf(mockk<LinkAddress> { every { address } returns mockk<Inet4Address>() })
        val cellularNetwork = mockk<Network>()
        val cellularNetworkLinkProperties = mockk<LinkProperties>()
        every { cellularNetworkLinkProperties.linkAddresses } returns
            listOf(mockk<LinkAddress> { every { address } returns mockk<Inet4Address>() })
        val mockProtect = mockk<(socket: DatagramSocket) -> Boolean>(relaxed = true)

        every { connectivityManager.defaultNetworkEvents() } returns
            callbackFlow {
                send(NetworkEvent.Available(wifiNetwork))
                send(NetworkEvent.LinkPropertiesChanged(wifiNetwork, wifiNetworkLinkProperties))
                delay(5.seconds)
                send(NetworkEvent.Lost(wifiNetwork))
                // We will have no network for a little time until cellular chip is on.
                delay(150.milliseconds)
                send(NetworkEvent.Available(cellularNetwork))
                send(
                    NetworkEvent.LinkPropertiesChanged(
                        cellularNetwork,
                        cellularNetworkLinkProperties,
                    )
                )
                awaitClose {}
            }

        connectivityManager.hasInternetConnectivity(mockProtect).test {
            // We should always only see us being online, small offline state is caught by debounce
            assertEquals(Connectivity.Status(ipv4 = true, ipv6 = false), awaitItem())
            expectNoEvents()
        }
    }

    /** User slow roaming from WiFi to Cellular. */
    @Test
    fun slowRoamingFromWifiToCellular() = runTest {
        val wifiNetwork = mockk<Network>()
        val wifiNetworkLinkProperties = mockk<LinkProperties>()
        every { wifiNetworkLinkProperties.linkAddresses } returns
            listOf(mockk<LinkAddress> { every { address } returns mockk<Inet6Address>() })
        val cellularNetwork = mockk<Network>()
        val cellularNetworkLinkProperties = mockk<LinkProperties>()
        every { cellularNetworkLinkProperties.linkAddresses } returns
            listOf(mockk<LinkAddress> { every { address } returns mockk<Inet6Address>() })
        val mockProtect = mockk<(socket: DatagramSocket) -> Boolean>(relaxed = true)

        every { connectivityManager.defaultNetworkEvents() } returns
            callbackFlow {
                send(NetworkEvent.Available(wifiNetwork))
                send(NetworkEvent.LinkPropertiesChanged(wifiNetwork, wifiNetworkLinkProperties))
                delay(5.seconds)
                send(NetworkEvent.Lost(wifiNetwork))
                // We will have no network for a little time until cellular chip is on.
                delay(500.milliseconds)
                send(NetworkEvent.Available(cellularNetwork))
                send(
                    NetworkEvent.LinkPropertiesChanged(
                        cellularNetwork,
                        cellularNetworkLinkProperties,
                    )
                )
                awaitClose {}
            }

        connectivityManager.hasInternetConnectivity(protect = mockProtect).test {
            // Wifi is online
            assertEquals(Connectivity.Status(false, true), awaitItem())
            // We didn't get any network within debounce time, so we are offline
            assertEquals(Connectivity.Status(false, false), awaitItem())
            // Cellular network is online
            assertEquals(Connectivity.Status(false, true), awaitItem())
            expectNoEvents()
        }
    }

    /** Switching between networks with different configurations. */
    @Test
    fun roamingFromWifiWithIpv6OnlyToWifiWithIpv4Only() = runTest {
        val ipv6Network = mockk<Network>()
        val ipv6NetworkLinkProperties = mockk<LinkProperties>()
        every { ipv6NetworkLinkProperties.linkAddresses } returns
            listOf(mockk<LinkAddress> { every { address } returns mockk<Inet4Address>() })
        val ipv4Network = mockk<Network>()
        val ipv4NetworkkLinkProperties = mockk<LinkProperties>()
        every { ipv4NetworkkLinkProperties.linkAddresses } returns
            listOf(mockk<LinkAddress> { every { address } returns mockk<Inet6Address>() })
        val mockProtect = mockk<(socket: DatagramSocket) -> Boolean>(relaxed = true)

        every { connectivityManager.defaultNetworkEvents() } returns
            callbackFlow {
                send(NetworkEvent.Available(ipv6Network))
                send(NetworkEvent.LinkPropertiesChanged(ipv6Network, ipv6NetworkLinkProperties))
                delay(5.seconds)
                send(NetworkEvent.Lost(ipv6Network))
                delay(100.milliseconds)
                send(NetworkEvent.Available(ipv4Network))
                send(NetworkEvent.LinkPropertiesChanged(ipv4Network, ipv4NetworkkLinkProperties))
                awaitClose {}
            }

        connectivityManager.hasInternetConnectivity(protect = mockProtect).test {
            // Ipv4 network is online
            assertEquals(Connectivity.Status(true, false), awaitItem())
            // Ipv6 network is online
            assertEquals(Connectivity.Status(false, true), awaitItem())
            expectNoEvents()
        }
    }

    /** Vpn network should NOT check link properties but should rather use socket implementation */
    @Test
    fun checkVpnNetworkUsingSocketImplementation() = runTest {
        val vpnNetwork = mockk<Network>()
        val capabilities = mockk<NetworkCapabilities>()
        every { capabilities.hasCapability(NetworkCapabilities.NET_CAPABILITY_NOT_VPN) } returns
            false
        val mockProtect = mockk<(socket: DatagramSocket) -> Boolean>()
        every { IpUtils.hasIPv4(any()) } returns true
        every { IpUtils.hasIPv6(any()) } returns true

        every { connectivityManager.defaultNetworkEvents() } returns
            callbackFlow {
                send(NetworkEvent.Available(vpnNetwork))
                send(NetworkEvent.CapabilitiesChanged(vpnNetwork, capabilities))
                awaitClose {}
            }

        connectivityManager.hasInternetConnectivity(protect = mockProtect).test {
            // Network is online
            assertEquals(Connectivity.Status(true, true), awaitItem())
        }

        verify(exactly = 1) { IpUtils.hasIPv4(any()) }
        verify(exactly = 1) { IpUtils.hasIPv6(any()) }
    }

    companion object {
        private const val CONNECTIVITY_MANAGER_UTIL_CLASS =
            "net.mullvad.talpid.util.ConnectivityManagerUtilKt"
    }
}
