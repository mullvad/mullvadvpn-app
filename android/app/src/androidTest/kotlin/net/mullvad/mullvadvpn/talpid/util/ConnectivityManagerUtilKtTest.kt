package net.mullvad.mullvadvpn.talpid.util

import android.net.ConnectivityManager
import android.net.Network
import app.cash.turbine.test
import io.mockk.every
import io.mockk.mockk
import io.mockk.mockkStatic
import kotlin.test.assertEquals
import kotlin.time.Duration.Companion.milliseconds
import kotlin.time.Duration.Companion.seconds
import kotlinx.coroutines.channels.awaitClose
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.callbackFlow
import kotlinx.coroutines.test.runTest
import net.mullvad.talpid.util.NetworkEvent
import net.mullvad.talpid.util.hasInternetConnectivity
import net.mullvad.talpid.util.networkEvents
import net.mullvad.talpid.util.networksWithInternetConnectivity
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
        val network = mockk<Network>()
        every { connectivityManager.networksWithInternetConnectivity() } returns setOf(network)
        every { connectivityManager.networkEvents(any()) } returns
            callbackFlow {
                delay(100.milliseconds) // Simulate connectivity listener being a bit slow
                send(NetworkEvent.Available(network))
                awaitClose {}
            }

        connectivityManager.hasInternetConnectivity().test {
            // Since initial state and listener both return `true` within debounce we only see one
            // event
            assertEquals(true, awaitItem())
            expectNoEvents()
        }
    }

    /** User being offline, the listener should emit once with `false` */
    @Test
    fun userIsOffline() = runTest {
        every { connectivityManager.networksWithInternetConnectivity() } returns setOf()
        every { connectivityManager.networkEvents(any()) } returns callbackFlow { awaitClose {} }

        connectivityManager.hasInternetConnectivity().test {
            // Initially offline and no network events, so we should get a single `false` event
            assertEquals(false, awaitItem())
            expectNoEvents()
        }
    }

    /** User starting offline and then turning on a online after a while */
    @Test
    fun initiallyOfflineThenBecomingOnline() = runTest {
        every { connectivityManager.networksWithInternetConnectivity() } returns emptySet()
        every { connectivityManager.networkEvents(any()) } returns
            callbackFlow {
                // Simulate offline for a little while
                delay(5.seconds)
                // Then become online
                send(NetworkEvent.Available(mockk()))
                awaitClose {}
            }

        connectivityManager.hasInternetConnectivity().test {
            assertEquals(false, awaitItem())
            assertEquals(true, awaitItem())
            expectNoEvents()
        }
    }

    /** User starting online and then becoming offline after 5 seconds */
    @Test
    fun initiallyOnlineAndThenTurningBecomingOffline() = runTest {
        val network = mockk<Network>()
        every { connectivityManager.networksWithInternetConnectivity() } returns setOf(network)
        every { connectivityManager.networkEvents(any()) } returns
            callbackFlow {
                // Starting as online
                send(NetworkEvent.Available(network))
                delay(5.seconds)
                // Then becoming offline
                send(NetworkEvent.Lost(network))
                awaitClose {}
            }

        connectivityManager.hasInternetConnectivity().test {
            assertEquals(true, awaitItem())
            assertEquals(false, awaitItem())
            expectNoEvents()
        }
    }

    /**
     * User turning on Airplane mode as our connectivity listener starts so we never get any
     * onAvailable event from our listener. Initial value will be `true`, followed by no
     * `networkEvent` and then turning on network again after 5 seconds
     */
    @Test
    fun incorrectInitialValueThenBecomingOnline() = runTest {
        every { connectivityManager.networksWithInternetConnectivity() } returns setOf(mockk())
        every { connectivityManager.networkEvents(any()) } returns
            callbackFlow {
                delay(5.seconds)
                send(NetworkEvent.Available(mockk()))
                awaitClose {}
            }

        connectivityManager.hasInternetConnectivity().test {
            // Initial value is connected
            assertEquals(true, awaitItem())
            // Debounce time has passed, and we never received any network events, so we are offline
            assertEquals(false, awaitItem())
            // Network is back online
            assertEquals(true, awaitItem())
            expectNoEvents()
        }
    }

    /** User roaming from cellular to WiFi. This behavior has been recorded on a Pixel 8 */
    @Test
    fun roamingFromCellularToWifi() = runTest {
        val wifiNetwork = mockk<Network>()
        val cellularNetwork = mockk<Network>()

        every { connectivityManager.networksWithInternetConnectivity() } returns
            setOf(cellularNetwork)
        every { connectivityManager.networkEvents(any()) } returns
            callbackFlow {
                send(NetworkEvent.Available(cellularNetwork))
                delay(5.seconds)
                // Turning on WiFi, we'll have duplicate networks until phone decides to turn of
                // cellular
                send(NetworkEvent.Available(wifiNetwork))
                delay(30.seconds)
                // Phone turning off cellular network
                send(NetworkEvent.Lost(cellularNetwork))
                awaitClose {}
            }

        connectivityManager.hasInternetConnectivity().test {
            // We should always only see us being online
            assertEquals(true, awaitItem())
            expectNoEvents()
        }
    }

    /** User roaming from WiFi to Cellular. This behavior has been recorded on a Pixel 8 */
    @Test
    fun roamingFromWifiToCellular() = runTest {
        val wifiNetwork = mockk<Network>()
        val cellularNetwork = mockk<Network>()

        every { connectivityManager.networksWithInternetConnectivity() } returns setOf(wifiNetwork)
        every { connectivityManager.networkEvents(any()) } returns
            callbackFlow {
                send(NetworkEvent.Available(wifiNetwork))
                delay(5.seconds)
                send(NetworkEvent.Lost(wifiNetwork))
                // We will have no network for a little time until cellular chip is on.
                delay(150.milliseconds)
                send(NetworkEvent.Available(cellularNetwork))
                awaitClose {}
            }

        connectivityManager.hasInternetConnectivity().test {
            // We should always only see us being online, small offline state is caught by debounce
            assertEquals(true, awaitItem())
            expectNoEvents()
        }
    }

    /** User slow roaming from WiFi to Cellular. */
    @Test
    fun slowRoamingFromWifiToCellular() = runTest {
        val wifiNetwork = mockk<Network>()
        val cellularNetwork = mockk<Network>()

        every { connectivityManager.networksWithInternetConnectivity() } returns setOf(wifiNetwork)
        every { connectivityManager.networkEvents(any()) } returns
            callbackFlow {
                send(NetworkEvent.Available(wifiNetwork))
                delay(5.seconds)
                send(NetworkEvent.Lost(wifiNetwork))
                // We will have no network for a little time until cellular chip is on.
                delay(500.milliseconds)
                send(NetworkEvent.Available(cellularNetwork))
                awaitClose {}
            }

        connectivityManager.hasInternetConnectivity().test {
            // Wifi is online
            assertEquals(true, awaitItem())
            // We didn't get any network within debounce time, so we are offline
            assertEquals(false, awaitItem())
            // Cellular network is online
            assertEquals(true, awaitItem())
            expectNoEvents()
        }
    }

    companion object {
        private const val CONNECTIVITY_MANAGER_UTIL_CLASS =
            "net.mullvad.talpid.util.ConnectivityManagerUtilKt"
    }
}
