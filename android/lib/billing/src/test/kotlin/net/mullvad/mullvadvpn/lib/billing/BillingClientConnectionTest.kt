package net.mullvad.mullvadvpn.lib.billing

import android.content.Context
import com.android.billingclient.api.BillingClient
import com.android.billingclient.api.BillingClient.BillingResponseCode
import com.android.billingclient.api.BillingClientStateListener
import com.android.billingclient.api.BillingResult
import com.android.billingclient.api.PendingPurchasesParams
import com.android.billingclient.api.PurchasesUpdatedListener
import io.mockk.Runs
import io.mockk.every
import io.mockk.just
import io.mockk.mockk
import io.mockk.mockkStatic
import io.mockk.slot
import io.mockk.unmockkAll
import io.mockk.verify
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertNotEquals
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.async
import kotlinx.coroutines.test.UnconfinedTestDispatcher
import kotlinx.coroutines.test.runCurrent
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.lib.billing.model.BillingException
import org.junit.jupiter.api.AfterEach
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.assertThrows

class BillingClientConnectionTest {

    private val context: Context = mockk()
    private val purchaseListener: PurchasesUpdatedListener = mockk()
    private val mockBillingClient: BillingClient = mockk()
    private val mockClientBuilder: BillingClient.Builder = mockk()

    private lateinit var connection: BillingClientConnection

    @OptIn(ExperimentalCoroutinesApi::class)
    @BeforeEach
    fun setup() {
        mockkStatic(BillingClient::class)
        mockkStatic(PendingPurchasesParams::class)

        val pendingBuilder: PendingPurchasesParams.Builder = mockk()
        every { PendingPurchasesParams.newBuilder() } returns pendingBuilder
        every { pendingBuilder.enableOneTimeProducts() } returns pendingBuilder
        every { pendingBuilder.build() } returns mockk()

        every { BillingClient.newBuilder(any()) } returns mockClientBuilder
        every { mockClientBuilder.setListener(any()) } returns mockClientBuilder
        every { mockClientBuilder.enablePendingPurchases(any<PendingPurchasesParams>()) } returns
            mockClientBuilder
        every { mockClientBuilder.build() } answers { mockBillingClient }

        every { mockBillingClient.connectionState } returns
            BillingClient.ConnectionState.DISCONNECTED

        connection =
            BillingClientConnection(
                context,
                purchaseListener,
                connectionStateDispatcher = UnconfinedTestDispatcher(),
            )
    }

    @AfterEach
    fun tearDown() {
        unmockkAll()
    }

    @Test
    fun `getClient should return client when billing setup finishes with OK`() = runTest {
        // Arrange
        every { mockBillingClient.startConnection(any()) } answers
            {
                firstArg<BillingClientStateListener>()
                    .onBillingSetupFinished(mockConnectionResultOK)
            }

        // Act
        val result = connection.getClient()

        // Assert
        assertEquals(mockBillingClient, result)
    }

    @Test
    fun `getClient should throw billing exception when setup fails`(): Unit = runTest {
        // Arrange
        every { mockBillingClient.startConnection(any()) } answers
            {
                firstArg<BillingClientStateListener>()
                    .onBillingSetupFinished(mockConnectionResultError)
            }

        // Act, Assert
        val exception = assertThrows<BillingException> { connection.getClient() }
        assertEquals("Could not setup billing client", exception.message)
    }

    @Test
    fun `getClient should throw exception when service disconnects while connecting`(): Unit =
        runTest {
            // Arrange
            every { mockBillingClient.startConnection(any()) } answers
                {
                    firstArg<BillingClientStateListener>().onBillingServiceDisconnected()
                }

            // Act, Assert
            val exception = assertThrows<BillingException> { connection.getClient() }
            assertEquals("Billing client is disconnected", exception.message)
        }

    @Test
    fun `when getClient is called several times ensure only one start connection is called`() =
        runTest {
            // Arrange
            every { mockBillingClient.startConnection(any()) } answers
                {
                    firstArg<BillingClientStateListener>()
                        .onBillingSetupFinished(mockConnectionResultOK)
                }

            // Act
            val firstClient = connection.getClient()
            every { mockBillingClient.connectionState } returns
                BillingClient.ConnectionState.CONNECTED
            // If we get a call to client builder again we will return another mock
            every { mockClientBuilder.build() } returns mockk()
            val secondClient = connection.getClient()

            // Assert
            assertEquals(mockBillingClient, firstClient)
            assertEquals(mockBillingClient, secondClient)
            verify(exactly = 1) { mockClientBuilder.build() }
            verify(exactly = 1) { mockBillingClient.startConnection(any()) }
        }

    @Test
    fun `getClient should reconnect with the same client after it was disconnected`() = runTest {
        // Arrange
        every { mockBillingClient.startConnection(any()) } answers
            {
                firstArg<BillingClientStateListener>()
                    .onBillingSetupFinished(mockConnectionResultOK)
            }

        // Act
        val firstClient = connection.getClient()
        every { mockBillingClient.connectionState } returns
            BillingClient.ConnectionState.DISCONNECTED
        val secondClient = connection.getClient()

        assertEquals(firstClient, secondClient)
        verify(exactly = 2) { mockBillingClient.startConnection(any()) }
        verify(exactly = 1) { mockClientBuilder.build() }
    }

    @Test
    fun `getClient should create a new client if the previous one was closed`() = runTest {
        // Arrange
        every { mockBillingClient.startConnection(any()) } answers
            {
                firstArg<BillingClientStateListener>()
                    .onBillingSetupFinished(mockConnectionResultOK)
            }
        val newMockBillingClient: BillingClient = mockk()
        every { newMockBillingClient.startConnection(any()) } answers
            {
                firstArg<BillingClientStateListener>()
                    .onBillingSetupFinished(mockConnectionResultOK)
            }
        every { newMockBillingClient.connectionState } returns
            BillingClient.ConnectionState.DISCONNECTED
        every { mockBillingClient.endConnection() } just Runs

        // Act
        val firstClient = connection.getClient()
        connection.endConnection()
        every { mockBillingClient.connectionState } returns BillingClient.ConnectionState.CLOSED
        every { mockClientBuilder.build() } returns newMockBillingClient
        val secondClient = connection.getClient()

        assertNotEquals(firstClient, secondClient)
        verify(exactly = 2) { mockClientBuilder.build() }
    }

    @OptIn(ExperimentalCoroutinesApi::class)
    @Test
    fun `callbacks from an old client should be ignored`() = runTest {
        // Arrange
        val oldListener = slot<BillingClientStateListener>()
        every { mockBillingClient.startConnection(capture(oldListener)) } answers
            {
                firstArg<BillingClientStateListener>()
                    .onBillingSetupFinished(mockConnectionResultOK)
            }
        every { mockBillingClient.endConnection() } just Runs

        val newMockBillingClient: BillingClient = mockk()
        val newListener = slot<BillingClientStateListener>()
        every { newMockBillingClient.connectionState } returns
            BillingClient.ConnectionState.DISCONNECTED
        every { newMockBillingClient.startConnection(capture(newListener)) } just Runs

        // Act
        connection.getClient()
        connection.endConnection()
        every { mockClientBuilder.build() } returns newMockBillingClient

        val deferred = async { connection.getClient() }
        runCurrent()
        oldListener.captured.onBillingSetupFinished(mockConnectionResultOK)
        oldListener.captured.onBillingServiceDisconnected()
        runCurrent()

        // Assert
        assertFalse(deferred.isCompleted)

        newListener.captured.onBillingSetupFinished(mockConnectionResultOK)
        assertEquals(newMockBillingClient, deferred.await())
    }

    @Test
    fun `endConnection should call client endConnection`() = runTest {
        // Arrange
        every { mockBillingClient.endConnection() } just Runs
        every { mockBillingClient.startConnection(any()) } answers
            {
                firstArg<BillingClientStateListener>()
                    .onBillingSetupFinished(mockConnectionResultOK)
            }

        connection.getClient()
        connection.endConnection()

        verify { mockBillingClient.endConnection() }
    }

    @OptIn(ExperimentalCoroutinesApi::class)
    @Test
    fun `if end connection is called during a get client call, that call should throw an exception`() =
        runTest {
            // Arrange
            every { mockBillingClient.startConnection(any()) } answers {}
            every { mockBillingClient.endConnection() } just Runs

            // Act
            val deferred = async { runCatching { connection.getClient() } }
            runCurrent()
            connection.endConnection()
            runCurrent()

            // Assert
            val exception = deferred.await().exceptionOrNull()
            assertEquals("Billing client is closed", exception?.message)
        }

    @Test
    fun `after a failed call due to failed billing setup next call should try to setup a connection again`() =
        runTest {
            // Arrange
            every { mockBillingClient.startConnection(any()) } answers
                {
                    firstArg<BillingClientStateListener>()
                        .onBillingSetupFinished(mockConnectionResultError)
                } andThenAnswer
                {
                    firstArg<BillingClientStateListener>()
                        .onBillingSetupFinished(mockConnectionResultOK)
                }

            // Act, Assert
            assertThrows<BillingException> { connection.getClient() }
            connection.getClient()

            verify(exactly = 1) { mockClientBuilder.build() }
            verify(exactly = 2) { mockBillingClient.startConnection(any()) }
        }

    @Test
    fun `if the client connects after a failed call the client should be returned`() = runTest {
        // Arrange
        every { mockBillingClient.startConnection(any()) } answers
            {
                firstArg<BillingClientStateListener>()
                    .onBillingSetupFinished(mockConnectionResultError)
            }

        // Act, Assert
        assertThrows<BillingException> { connection.getClient() }
        every { mockBillingClient.connectionState } returns BillingClient.ConnectionState.CONNECTED
        val result = connection.getClient()
        assertEquals(mockBillingClient, result)
    }

    @Test
    fun `getClient should timeout if the setup callback never arrives`() = runTest {
        // Arrange
        every { mockBillingClient.startConnection(any()) } just Runs

        // Act, Assert
        val exception = assertThrows<BillingException> { connection.getClient() }
        assertEquals("Timeout waiting for billing client to connect", exception.message)
    }

    companion object {
        private val mockConnectionResultOK: BillingResult = mockk {
            every { responseCode } returns BillingResponseCode.OK
            every { debugMessage } returns "OK"
        }

        private val mockConnectionResultError: BillingResult = mockk {
            every { responseCode } returns BillingResponseCode.ERROR
            every { debugMessage } returns "Could not setup billing client"
        }
    }
}
