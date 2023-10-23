package net.mullvad.mullvadvpn.usecase

import app.cash.turbine.test
import io.mockk.MockKAnnotations
import io.mockk.every
import io.mockk.mockk
import io.mockk.unmockkAll
import kotlin.test.assertEquals
import kotlin.test.assertTrue
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.lib.common.test.TestCoroutineRule
import net.mullvad.mullvadvpn.model.TunnelState
import net.mullvad.mullvadvpn.repository.InAppNotification
import net.mullvad.mullvadvpn.ui.serviceconnection.ConnectionProxy
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionContainer
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManager
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionState
import net.mullvad.talpid.tunnel.ActionAfterDisconnect
import net.mullvad.talpid.tunnel.ErrorState
import net.mullvad.talpid.util.EventNotifier
import org.junit.After
import org.junit.Before
import org.junit.Rule
import org.junit.Test

class TunnelStateNotificationUseCaseTest {
    @get:Rule val testCoroutineRule = TestCoroutineRule()

    private val mockServiceConnectionManager: ServiceConnectionManager = mockk()
    private val mockServiceConnectionContainer: ServiceConnectionContainer = mockk()
    private val mockConnectionProxy: ConnectionProxy = mockk()

    private val serviceConnectionState =
        MutableStateFlow<ServiceConnectionState>(ServiceConnectionState.Disconnected)
    private lateinit var tunnelStateNotificationUseCase: TunnelStateNotificationUseCase

    private val eventNotifierTunnelUiState = EventNotifier<TunnelState>(TunnelState.Disconnected)

    @Before
    fun setup() {
        MockKAnnotations.init(this)
        every { mockConnectionProxy.onUiStateChange } returns eventNotifierTunnelUiState

        every { mockServiceConnectionManager.connectionState } returns serviceConnectionState
        every { mockServiceConnectionContainer.connectionProxy } returns mockConnectionProxy

        tunnelStateNotificationUseCase =
            TunnelStateNotificationUseCase(serviceConnectionManager = mockServiceConnectionManager)
    }

    @After
    fun teardown() {
        unmockkAll()
    }

    @Test
    fun `ensure notifications are empty by default`() = runTest {
        // Arrange, Act, Assert
        tunnelStateNotificationUseCase.notifications().test { assertTrue { awaitItem().isEmpty() } }
    }

    @Test
    fun `ensure TunnelState with error will produce TunnelStateError notification`() = runTest {
        tunnelStateNotificationUseCase.notifications().test {
            // Arrange, Act
            assertEquals(emptyList(), awaitItem())
            serviceConnectionState.value =
                ServiceConnectionState.ConnectedReady(mockServiceConnectionContainer)
            val errorState: ErrorState = mockk()
            eventNotifierTunnelUiState.notify(TunnelState.Error(errorState))

            // Assert
            assertEquals(listOf(InAppNotification.TunnelStateError(errorState)), awaitItem())
        }
    }

    @Test
    fun `ensure disconnecting TunnelState with blocking will produce TunnelStateBlocked notification`() =
        runTest {
            tunnelStateNotificationUseCase.notifications().test {
                // Arrange, Act
                assertEquals(emptyList(), awaitItem())
                serviceConnectionState.value =
                    ServiceConnectionState.ConnectedReady(mockServiceConnectionContainer)
                eventNotifierTunnelUiState.notify(
                    TunnelState.Disconnecting(ActionAfterDisconnect.Block)
                )

                // Assert
                assertEquals(listOf(InAppNotification.TunnelStateBlocked), awaitItem())
            }
        }
}
