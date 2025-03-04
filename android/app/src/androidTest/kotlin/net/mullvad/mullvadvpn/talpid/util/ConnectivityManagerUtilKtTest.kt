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
                delay(100.milliseconds)
                send(NetworkEvent.Available(network))
                awaitClose {}
            }

        connectivityManager.hasInternetConnectivity().test {
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
                delay(5.seconds)
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
                send(NetworkEvent.Available(network))
                delay(5.seconds)
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
     * User turning of Airplane mode as our connectivity starts so we never get any onAvailable
     * event from our listener. Initial value will be `true`, followed by no `networkEvent` and then
     * turning on network again after 5 seconds
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
            assertEquals(true, awaitItem())
            assertEquals(false, awaitItem())
            assertEquals(true, awaitItem())
            expectNoEvents()
        }
    }

    companion object {
        private const val CONNECTIVITY_MANAGER_UTIL_CLASS =
            "net.mullvad.talpid.util.ConnectivityManagerUtilKt"
    }
}
