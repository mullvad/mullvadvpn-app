package net.mullvad.mullvadvpn.talpid.util

import android.net.ConnectivityManager
import android.net.LinkAddress
import android.net.LinkProperties
import android.net.Network
import android.net.NetworkCapabilities
import app.cash.turbine.test
import io.mockk.every
import io.mockk.mockk
import io.mockk.mockkStatic
import io.mockk.verify
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
import net.mullvad.talpid.util.NetworkEvent
import net.mullvad.talpid.util.UnderlyingConnectivityStatusResolver
import net.mullvad.talpid.util.defaultNetworkEvents
import net.mullvad.talpid.util.hasInternetConnectivity
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test

class ConnectivityManagerUtilKtTest {
    private val connectivityManager = mockk<ConnectivityManager>()

    @BeforeEach
    fun setup() {
        mockkStatic(CONNECTIVITY_MANAGER_UTIL_CLASS)
    }

    /** User being online, the listener should emit once with `true` */
    @Test
    fun userIsOnline() = runTest {
        val network = mockk<Network>(relaxed = true)
        val linkProperties = mockLinkProperties(true, true)
        val mockResolver = mockk<UnderlyingConnectivityStatusResolver>()
        every { connectivityManager.defaultNetworkEvents() } returns
            callbackFlow {
                delay(100.milliseconds) // Simulate connectivity listener being a bit slow
                send(NetworkEvent.Available(network))
                delay(100.milliseconds) // Simulate connectivity listener being a bit slow
                send(NetworkEvent.LinkPropertiesChanged(network, linkProperties))
                awaitClose {}
            }

        connectivityManager.hasInternetConnectivity(mockResolver).test {
            // Since initial state and listener both return `true` within debounce we only see one
            // event
            assertEquals(Connectivity.Status(true, true), awaitItem())
            expectNoEvents()
        }
    }

    /** User being offline, the listener should emit once with `false` */
    @Test
    fun userIsOffline() = runTest {
        val mockResolver = mockk<UnderlyingConnectivityStatusResolver>()
        every { connectivityManager.defaultNetworkEvents() } returns callbackFlow { awaitClose {} }

        connectivityManager.hasInternetConnectivity(mockResolver).test {
            // Initially offline and no network events, so we should get a single `false` event
            assertEquals(Connectivity.Status(false, false), awaitItem())
            expectNoEvents()
        }
    }

    /** User starting offline and then turning on a online after a while */
    @Test
    fun initiallyOfflineThenBecomingOnline() = runTest {
        val network = mockk<Network>()
        val linkProperties = mockLinkProperties(true, true)
        val mockResolver = mockk<UnderlyingConnectivityStatusResolver>()
        every { connectivityManager.defaultNetworkEvents() } returns
            callbackFlow {
                // Simulate offline for a little while
                delay(5.seconds)
                // Then become online
                send(NetworkEvent.Available(mockk()))
                send(NetworkEvent.LinkPropertiesChanged(network, linkProperties))
                awaitClose {}
            }

        connectivityManager.hasInternetConnectivity(mockResolver).test {
            assertEquals(Connectivity.Status(false, false), awaitItem())
            assertEquals(Connectivity.Status(true, true), awaitItem())
            expectNoEvents()
        }
    }

    /** User starting online and then becoming offline after 5 seconds */
    @Test
    fun initiallyOnlineAndThenTurningBecomingOffline() = runTest {
        val network = mockk<Network>()
        val linkProperties = mockLinkProperties(true, true)

        val mockResolver = mockk<UnderlyingConnectivityStatusResolver>()
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

        connectivityManager.hasInternetConnectivity(mockResolver).test {
            assertEquals(Connectivity.Status(true, true), awaitItem())
            assertEquals(Connectivity.Status(false, false), awaitItem())
            expectNoEvents()
        }
    }

    /** User roaming from cellular to WiFi. This behavior has been recorded on a Pixel 8 */
    @Test
    fun roamingFromCellularToWifi() = runTest {
        val wifiNetwork = mockk<Network>()
        val wifiNetworkLinkProperties = mockLinkProperties(true, false)
        val cellularNetwork = mockk<Network>()
        val cellularNetworkLinkProperties = mockLinkProperties(true, false)
        val mockResolver = mockk<UnderlyingConnectivityStatusResolver>()

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

        connectivityManager.hasInternetConnectivity(mockResolver).test {
            // We should always only see us being online
            assertEquals(Connectivity.Status(ipv4 = true, ipv6 = false), awaitItem())
            expectNoEvents()
        }
    }

    /** User roaming from WiFi to Cellular. This behavior has been recorded on a Pixel 8 */
    @Test
    fun roamingFromWifiToCellular() = runTest {
        val wifiNetwork = mockk<Network>()
        val wifiNetworkLinkProperties = mockLinkProperties(true, false)
        val cellularNetwork = mockk<Network>()
        val cellularNetworkLinkProperties = mockLinkProperties(true, false)
        val mockResolver = mockk<UnderlyingConnectivityStatusResolver>()

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

        connectivityManager.hasInternetConnectivity(mockResolver).test {
            // We should always only see us being online, small offline state is caught by debounce
            assertEquals(Connectivity.Status(ipv4 = true, ipv6 = false), awaitItem())
            expectNoEvents()
        }
    }

    /** User slow roaming from WiFi to Cellular. */
    @Test
    fun slowRoamingFromWifiToCellular() = runTest {
        val wifiNetwork = mockk<Network>()
        val wifiNetworkLinkProperties = mockLinkProperties(false, true)
        val cellularNetwork = mockk<Network>()
        val cellularNetworkLinkProperties = mockLinkProperties(false, true)
        val mockResolver = mockk<UnderlyingConnectivityStatusResolver>()

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

        connectivityManager.hasInternetConnectivity(mockResolver).test {
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
        val ipv6NetworkLinkProperties = mockLinkProperties(false, true)
        val ipv4Network = mockk<Network>()
        val ipv4NetworkLinkProperties = mockLinkProperties(true, false)
        val mockResolver = mockk<UnderlyingConnectivityStatusResolver>()

        every { connectivityManager.defaultNetworkEvents() } returns
            callbackFlow {
                send(NetworkEvent.Available(ipv6Network))
                send(NetworkEvent.LinkPropertiesChanged(ipv6Network, ipv6NetworkLinkProperties))
                delay(5.seconds)
                send(NetworkEvent.Lost(ipv6Network))
                delay(100.milliseconds)
                send(NetworkEvent.Available(ipv4Network))
                send(NetworkEvent.LinkPropertiesChanged(ipv4Network, ipv4NetworkLinkProperties))
                awaitClose {}
            }

        connectivityManager.hasInternetConnectivity(mockResolver).test {
            // Ipv6 network is online
            assertEquals(Connectivity.Status(false, true), awaitItem())
            // Ipv4 network is online
            assertEquals(Connectivity.Status(true, false), awaitItem())
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
        val mockResolver = mockk<UnderlyingConnectivityStatusResolver>()
        every { mockResolver.currentStatus() } returns Connectivity.Status(true, true)

        every { connectivityManager.defaultNetworkEvents() } returns
            callbackFlow {
                send(NetworkEvent.Available(vpnNetwork))
                send(NetworkEvent.CapabilitiesChanged(vpnNetwork, capabilities))
                awaitClose {}
            }

        connectivityManager.hasInternetConnectivity(mockResolver).test {
            // Network is online
            assertEquals(Connectivity.Status(true, true), awaitItem())
        }

        verify(exactly = 1) { mockResolver.currentStatus() }
    }

    private fun mockLinkProperties(ipv4: Boolean, ipv6: Boolean) =
        mockk<LinkProperties> {
            val linkAddresses = buildList {
                if (ipv4) {
                    val linkIpv4Address: LinkAddress = mockk()
                    val ipv4Address: Inet4Address = mockk()
                    every { linkIpv4Address.address } returns ipv4Address
                    add(linkIpv4Address)
                }
                if (ipv6) {
                    val linkIpv6Address: LinkAddress = mockk()
                    val ipv6Address: Inet6Address = mockk()
                    every { linkIpv6Address.address } returns ipv6Address
                    add(linkIpv6Address)
                }
            }

            every { this@mockk.linkAddresses } returns linkAddresses
        }

    companion object {
        private const val CONNECTIVITY_MANAGER_UTIL_CLASS =
            "net.mullvad.talpid.util.ConnectivityManagerUtilKt"
    }
}
